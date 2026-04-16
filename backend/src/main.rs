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
mod transactions;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Recurrency API server...");

    // Build database pool
    let db_pool = shared::db::create_pool().await?;

    // Run migrations
    shared::db::run_migrations(&db_pool).await?;

    tracing::info!("Database connected and migrations complete");

    // Build application
    let app = build_app(db_pool);

    // Start server - use PORT env var or default 8080 (dev) / 80 (prod)
    let port: u16 = std::env::var("APP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_app(db_pool: sqlx::PgPool) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Frontend dist path
    let frontend_dist = std::env::var("FRONTEND_DIST").unwrap_or_else(|_| "/frontend/dist".to_string());
    tracing::info!("Serving frontend from: {}", frontend_dist);

    let index_path = format!("{}/index.html", frontend_dist);
    if !std::path::Path::new(&index_path).exists() {
        tracing::error!("Frontend index.html not found at: {}", index_path);
        tracing::error!("Ensure FRONTEND_DIST points to a directory containing index.html");
        if let Ok(entries) = std::fs::read_dir(&frontend_dist) {
            tracing::error!("Contents of {}:", frontend_dist);
            for entry in entries.flatten() {
                tracing::error!("  - {}", entry.path().display());
            }
        } else {
            tracing::error!("Directory {} does not exist or is empty", frontend_dist);
        }
    }

    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Auth routes
        .route("/api/auth/register", post(auth::handlers::register))
        .route("/api/auth/login", post(auth::handlers::login))
        .route("/api/auth/verify", post(auth::handlers::verify))
        .route("/api/auth/verify-email", post(auth::handlers::verify_email))
        .route("/api/auth/resend-verification", post(auth::handlers::resend_verification))
        .route("/api/settings", get(auth::handlers::get_settings))
        .route("/api/settings", put(auth::handlers::update_settings))
        
        // Subscriptions routes
        .route("/api/subscriptions", post(subscriptions::handlers::create))
        .route("/api/subscriptions", get(subscriptions::handlers::list))
        .route("/api/subscriptions/{id}", get(subscriptions::handlers::get))
        .route("/api/subscriptions/{id}", put(subscriptions::handlers::update))
        .route("/api/subscriptions/{id}", delete(subscriptions::handlers::delete))
        .route("/api/subscriptions/upcoming", get(subscriptions::handlers::upcoming))
        .route("/api/subscriptions/calendar", get(subscriptions::handlers::calendar))
        
        // Transactions routes
        .route("/api/transactions", post(transactions::handlers::create))
        .route("/api/transactions", get(transactions::handlers::list))
        .route("/api/transactions/date-range", get(transactions::handlers::by_date_range))
        .route("/api/transactions/{id}", get(transactions::handlers::get))
        .route("/api/transactions/{id}", put(transactions::handlers::update))
        .route("/api/transactions/{id}", delete(transactions::handlers::delete))
        
        // Categories routes
        .route("/api/categories", post(categories::handlers::create))
        .route("/api/categories", get(categories::handlers::list))
        .route("/api/categories/with-counts", get(categories::handlers::list_with_counts))
        .route("/api/categories/{id}", get(categories::handlers::get))
        .route("/api/categories/{id}", put(categories::handlers::update))
        .route("/api/categories/{id}", delete(categories::handlers::delete))
        .route("/api/categories/default", put(categories::handlers::set_default))
        
        // Reports routes
        .route("/api/reports/summary", get(reports::handlers::get_report))
        
        // Notifications routes
        .route("/api/notifications", get(notifications::handlers::list))
        .route("/api/notifications/read-all", post(notifications::handlers::mark_all_read))
        .route("/api/notifications/{id}/read", post(notifications::handlers::mark_read))
        .route("/api/notifications/{id}", delete(notifications::handlers::delete))
        
        // Notification preferences routes
        .route("/api/settings/notifications", get(notifications::handlers::get_notification_preferences))
        .route("/api/settings/notifications", put(notifications::handlers::update_notification_preferences))
        
        // User preferences routes
        .route("/api/preferences", get(preferences::handlers::get_preferences))
        .route("/api/preferences", put(preferences::handlers::update_preferences))
        
        // Demo reset routes
        .route("/api/demo/status", get(demo::handlers::status))
        .route("/api/demo/init", get(demo::handlers::init))
        .route("/api/demo/reset", post(demo::handlers::reset))
        
        // Logo search route
        .route("/api/logos/search", get(logos::handlers::search))
        
        .with_state(db_pool)
        .layer(cors)
        // Serve frontend static files, fallback to index.html for SPA routing
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

    #[tokio::test]
    async fn test_health_check() {
        let result = health_check().await;
        assert_eq!(result, "OK");
    }
}