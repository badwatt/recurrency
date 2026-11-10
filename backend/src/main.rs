use recurrency_backend::{
    accounts, auth, categories, demo, logos, notifications, preferences, reports, shared,
    subscriptions, transactions,
};

use axum::http::HeaderValue;
use axum::{
    extract::Json,
    routing::{delete, get, post, put},
    Router,
};
use recurrency_backend::shared::app_utils::parse_port;
use std::net::SocketAddr;

use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    let default_level = if std::env::var("APP_ENV").unwrap_or_default() == "production" {
        "warn"
    } else {
        "info"
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| default_level.into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    tracing::info!("Starting Recurrency API server...");
    auth::service::init_jwt_secret().map_err(std::io::Error::other)?;
    let db_pool = shared::db::create_pool_with_retry().await?;
    shared::db::run_migrations(&db_pool).await?;
    tracing::info!("Database connected and migrations complete");

    let app = build_app(db_pool.clone());
    notifications::scheduler::spawn(db_pool);
    let port = parse_port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_app(db_pool: sqlx::PgPool) -> Router {
    use shared::app_state::AppState;

    let cors = cors_layer();

    let app_state = AppState::new(db_pool.clone());

    Router::new()
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
        .route("/api/auth/register", post(auth::handlers::register))
        .route("/api/auth/login", post(auth::handlers::login))
        .route("/api/auth/verify", get(auth::handlers::verify))
        .route("/api/auth/verify", post(auth::handlers::verify_post))
        .route("/api/auth/verify-email", post(auth::handlers::verify_email))
        .route("/api/auth/resend-verification", post(auth::handlers::resend_verification))
        .route("/api/settings", get(auth::handlers::get_settings))
        .route("/api/settings", put(auth::handlers::update_settings))
        .route("/api/settings/account", post(auth::handlers::delete_account))
        .route("/api/subscriptions", post(subscriptions::handlers::create))
        .route("/api/subscriptions", get(subscriptions::handlers::list))
        .route("/api/subscriptions/{id}", get(subscriptions::handlers::get))
        .route("/api/subscriptions/{id}", put(subscriptions::handlers::update))
        .route("/api/subscriptions/{id}", delete(subscriptions::handlers::delete))
        .route("/api/subscriptions/upcoming", get(subscriptions::handlers::upcoming))
        .route("/api/subscriptions/calendar", get(subscriptions::handlers::calendar))
        .route("/api/transactions", post(transactions::handlers::create))
        .route("/api/transactions", get(transactions::handlers::list))
        .route("/api/transactions/date-range", get(transactions::handlers::by_date_range))
        .route("/api/transactions/{id}", get(transactions::handlers::get))
        .route("/api/transactions/{id}", put(transactions::handlers::update))
        .route("/api/transactions/{id}", delete(transactions::handlers::delete))
        .route("/api/accounts", post(accounts::handlers::create))
        .route("/api/accounts", get(accounts::handlers::list))
        .route("/api/accounts/with-counts", get(accounts::handlers::list_with_counts))
        .route("/api/accounts/{id}", get(accounts::handlers::get))
        .route("/api/accounts/{id}", put(accounts::handlers::update))
        .route("/api/accounts/{id}", delete(accounts::handlers::delete))
        .route("/api/accounts/default", put(accounts::handlers::set_default))
        .route("/api/categories", post(categories::handlers::create))
        .route("/api/categories", get(categories::handlers::list))
        .route("/api/categories/with-counts", get(categories::handlers::list_with_counts))
        .route("/api/categories/{id}", get(categories::handlers::get))
        .route("/api/categories/{id}", put(categories::handlers::update))
        .route("/api/categories/{id}", delete(categories::handlers::delete))
        .route("/api/categories/default", put(categories::handlers::set_default))
        .route("/api/reports/summary", get(reports::handlers::get_report))
        .route("/api/notifications", get(notifications::handlers::list))
        .route("/api/notifications/read-all", post(notifications::handlers::mark_all_read))
        .route("/api/notifications/{id}/read", post(notifications::handlers::mark_read))
        .route("/api/notifications/{id}", delete(notifications::handlers::delete))
        .route("/api/settings/notifications", get(notifications::handlers::get_notification_preferences))
        .route("/api/settings/notifications", put(notifications::handlers::update_notification_preferences))
        .route("/api/settings/notification-channels", get(notifications::channel_handlers::list_channels))
        .route("/api/settings/notification-channels/{provider}", put(notifications::channel_handlers::update_channel))
        .route("/api/settings/notification-channels/{provider}/test", post(notifications::channel_handlers::test_channel))
        .route("/api/settings/summary-notification", get(notifications::channel_handlers::get_summary))
        .route("/api/settings/summary-notification", put(notifications::channel_handlers::update_summary))
        .route("/api/settings/summary-notification/test", post(notifications::channel_handlers::send_summary_now))
        .route("/api/preferences", get(preferences::handlers::get_preferences))
        .route("/api/preferences", put(preferences::handlers::update_preferences))
        .route("/api/demo/status", get(demo::handlers::status))
        .route("/api/demo/init", get(demo::handlers::init))
        .route("/api/demo/reset", post(demo::handlers::reset))
        .route("/api/logos/search", get(logos::handlers::search))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "OK",
        "smtp_configured": std::env::var("SMTP_HOST").is_ok()
    }))
}

/// Resolve the allowed CORS origin. `None` means permissive (`*`), which is
/// fine for a token-in-header API with no cookies. Set `CORS_ALLOWED_ORIGIN`
/// to restrict the app to a single origin in production.
fn resolve_cors_origin(value: Option<&str>) -> Result<Option<HeaderValue>, String> {
    match value.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(origin) => origin
            .parse::<HeaderValue>()
            .map(Some)
            .map_err(|_| format!("invalid CORS_ALLOWED_ORIGIN: {origin}")),
    }
}

fn cors_layer() -> CorsLayer {
    let origin = resolve_cors_origin(std::env::var("CORS_ALLOWED_ORIGIN").ok().as_deref())
        .expect("CORS_ALLOWED_ORIGIN must be a valid HTTP header value");

    match origin {
        Some(value) => CorsLayer::new()
            .allow_origin(value)
            .allow_methods(Any)
            .allow_headers(Any),
        None => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_structure() {
        let response = health_check().await;
        let body = response.0;
        assert_eq!(body["status"], "OK");
    }

    #[test]
    fn cors_origin_unset_is_permissive() {
        assert!(resolve_cors_origin(None).unwrap().is_none());
    }

    #[test]
    fn cors_origin_blank_is_permissive() {
        assert!(resolve_cors_origin(Some("   ")).unwrap().is_none());
    }

    #[test]
    fn cors_origin_valid_is_parsed() {
        assert!(
            resolve_cors_origin(Some("https://app.example.com"))
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn cors_origin_invalid_is_error() {
        assert!(resolve_cors_origin(Some("bad\norigin")).is_err());
    }
}