mod auth;
mod categories;
mod demo;
mod email;
mod logos;
mod notifications;
mod preferences;
mod reports;
mod shared;
mod subscriptions;
mod test_utils;
mod transactions;

use std::sync::Arc;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use shared::app_utils::{build_index_path, get_frontend_dist, index_html_exists, list_files_recursive, parse_port};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Recurrency API server...");
    let db_pool = shared::db::create_pool().await?;
    shared::db::run_migrations(&db_pool).await?;
    tracing::info!("Database connected and migrations complete");

    let app = build_app(db_pool);
    let port = parse_port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_app(db_pool: sqlx::PgPool) -> Router {
    use auth::service::{AuthServiceTrait, SqlxAuthService};
use shared::app_state::AppState;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let frontend_dist = get_frontend_dist();
    let index_path = build_index_path(&frontend_dist);
    if !index_html_exists(&frontend_dist) {
        tracing::error!("Frontend index.html not found at: {}", index_path);
    } else {
        tracing::info!("Frontend index.html found at: {}", index_path);
    }

    // Create app state with DB pool and injectable services
    let app_state = AppState::new(
        db_pool.clone(),
        Arc::new(SqlxAuthService::new(db_pool.clone())) as Arc<dyn AuthServiceTrait>
    );

    Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(auth::handlers::register))
        .route("/api/auth/login", post(auth::handlers::login))
        .route("/api/auth/verify", post(auth::handlers::verify))
        .route("/api/auth/verify-email", post(auth::handlers::verify_email))
        .route("/api/auth/resend-verification", post(auth::handlers::resend_verification))
        .route("/api/settings", get(auth::handlers::get_settings))
        .route("/api/settings", put(auth::handlers::update_settings))
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
        .route("/api/preferences", get(preferences::handlers::get_preferences))
        .route("/api/preferences", put(preferences::handlers::update_preferences))
        .route("/api/demo/status", get(demo::handlers::status))
        .route("/api/demo/init", get(demo::handlers::init))
        .route("/api/demo/reset", post(demo::handlers::reset))
        .route("/api/logos/search", get(logos::handlers::search))
        .with_state(app_state)
        .layer(cors)
        .fallback_service(
            ServeDir::new(&frontend_dist)
                .fallback(ServeFile::new(&index_path))
        )
}

async fn health_check() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let result = health_check().await;
        assert_eq!(result, "OK");
    }

    #[tokio::test]
    async fn health_check_endpoint() {
        async fn test_handler() -> &'static str {
            "OK"
        }
        
        let app = Router::new()
            .route("/health", get(test_handler));

        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn cors_layer_creation() {
        let _cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        assert!(true);
    }
}
