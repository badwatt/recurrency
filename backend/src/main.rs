use recurrency_backend::{
    accounts, auth, backup, categories, currency, debts, demo, logos, notifications, preferences,
    reports, shared, subscriptions, transactions,
};

use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderValue, StatusCode};
use axum::{
    extract::Json,
    routing::{delete, get, post, put},
    Router,
};
use recurrency_backend::cli::{self, Command};
use recurrency_backend::shared::app_state::AppState;
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
    let cli = match cli::parse(std::env::args().skip(1)) {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("{error}\n\n{}", cli::usage());
            std::process::exit(2);
        }
    };
    if cli.command == Command::Help {
        println!("{}", cli::usage());
        return Ok(());
    }
    if cli.command == Command::Version {
        println!("recurrency {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    init_logging();

    match cli.command {
        Command::Migrate => {
            let db_pool = shared::db::create_pool_with_retry_url(cli.database_url.as_deref()).await?;
            shared::db::run_migrations(&db_pool).await?;
            shared::db::shutdown_embedded().await;
            println!("Migrations complete.");
            return Ok(());
        }
        Command::Backup => {
            let database_url = cli
                .database_url
                .clone()
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .filter(|url| !url.trim().is_empty())
                .ok_or_else(|| {
                    std::io::Error::other(
                        "DATABASE_URL or --database-url is required for backups",
                    )
                })?;
            backup::cli::run(&database_url).await?;
            shared::db::shutdown_embedded().await;
            return Ok(());
        }
        Command::Doctor => {
            let db_pool = shared::db::create_pool_with_retry_url(cli.database_url.as_deref()).await?;
            let one: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&db_pool).await?;
            shared::db::shutdown_embedded().await;
            println!("recurrency {}: configuration OK (database={one})", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Command::Serve => {}
        Command::Help | Command::Version => unreachable!(),
    }

    tracing::info!("Starting Recurrency API server...");
    let db_pool = shared::db::create_pool_with_retry_url(cli.database_url.as_deref()).await?;
    shared::db::run_migrations(&db_pool).await?;
    auth::service::init_jwt_secret_from_pool(&db_pool)
        .await
        .map_err(std::io::Error::other)?;
    tracing::info!("Database connected and migrations complete");

    let enable_web = cli
        .enable_web
        .unwrap_or_else(|| !matches!(std::env::var("ENABLE_WEB").as_deref(), Ok("false" | "FALSE" | "0")));
    let app = build_app(db_pool.clone(), enable_web);
    notifications::scheduler::spawn(db_pool);
    let port = cli.port.unwrap_or_else(parse_port);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app);
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        result = server => result?,
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutdown signal received");
        }
        _ = terminate => {
            tracing::info!("Termination signal received");
        }
    }
    shared::db::shutdown_embedded().await;

    Ok(())
}

#[allow(unused_variables)]
fn build_app(db_pool: sqlx::PgPool, enable_web: bool) -> Router {
    let cors = cors_layer();

    let app_state = AppState::new(db_pool.clone());

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
        .route("/api/ready", get(readiness_check))
        .route("/api/currency/rates", get(currency::handlers::rates))
        .route("/api/currency/settings", get(currency::handlers::get_settings).put(currency::handlers::update_settings))
        .route("/api/backup", get(backup::handlers::export))
        .route(
            "/api/backup/import",
            post(backup::handlers::import).layer(DefaultBodyLimit::max(32 * 1024 * 1024)),
        )
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
        .route("/api/debts", post(debts::handlers::create))
        .route("/api/debts", get(debts::handlers::list))
        .route("/api/debts/{id}", get(debts::handlers::get))
        .route("/api/debts/{id}", put(debts::handlers::update))
        .route("/api/debts/{id}", delete(debts::handlers::delete))
        .route("/api/debts/{id}/payments", post(debts::handlers::record_payment))
        .route("/api/debts/{id}/payments", get(debts::handlers::payments))
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
        .route("/api/logos/search", get(logos::handlers::search));
    #[cfg(feature = "embedded-web")]
    let app = if enable_web {
        app.fallback(recurrency_backend::embedded_web::serve)
    } else {
        app
    };
    app.with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "OK",
        "smtp_configured": std::env::var("SMTP_HOST").is_ok()
    }))
}

/// Readiness verifies the database before a load balancer or container
/// healthcheck sends traffic to this instance.
async fn readiness_check(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "OK", "database": "ok"})),
        ),
        Err(error) => {
            tracing::warn!("Readiness check failed: {error}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"status": "UNAVAILABLE", "database": "unavailable"})),
            )
        }
    }
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
    async fn readiness_fails_when_database_is_unavailable() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(100))
            .connect_lazy("postgres://127.0.0.1:1/missing")
            .unwrap();
        let state = AppState::new(pool);
        let (status, body) = readiness_check(State(state)).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body.0["database"], "unavailable");
    }

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