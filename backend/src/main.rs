mod auth;
mod categories;
mod demo;
mod email;
mod notifications;
mod preferences;
mod reports;
mod shared;
mod subscriptions;
mod transactions;

use axum::{
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
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

    tracing::info!("Starting Recur API server...");

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
        
        .with_state(db_pool)
        .layer(cors)
        // SPA fallback - serve index.html for non-API routes
        .fallback(spa_fallback)
}

async fn health_check() -> &'static str {
    "OK"
}

// SPA fallback handler - serves index.html for non-API routes
async fn spa_fallback(req: Request) -> Response {
    // Get frontend dist path from environment
    let frontend_dist = std::env::var("FRONTEND_DIST").unwrap_or_else(|_| "/frontend/dist".to_string());
    
    let path = req.uri().path();
    
    // If it's an API route, return 404 (shouldn't happen due to route matching)
    if path.starts_with("/api/") {
        return (StatusCode::NOT_FOUND, "API endpoint not found").into_response();
    }
    
    // Try to serve the requested file
    let file_path = format!("{}{}", frontend_dist, path);
    if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
        let mime = if file_path.ends_with(".css") {
            "text/css"
        } else if file_path.ends_with(".js") {
            "application/javascript"
        } else if file_path.ends_with(".html") {
            "text/html"
        } else if file_path.ends_with(".svg") {
            "image/svg+xml"
        } else if file_path.ends_with(".png") {
            "image/png"
        } else if file_path.ends_with(".ico") {
            "image/x-icon"
        } else if file_path.ends_with(".json") {
            "application/json"
        } else if file_path.ends_with(".woff2") {
            "font/woff2"
        } else if file_path.ends_with(".woff") {
            "font/woff"
        } else {
            "application/octet-stream"
        };
        
        return Response::builder()
            .header("Content-Type", mime)
            .body(content.into())
            .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Error").into_response());
    }
    
    // Fallback to index.html for SPA routing
    let index_path = format!("{}/index.html", frontend_dist);
    match tokio::fs::read_to_string(&index_path).await {
        Ok(content) => Response::builder()
            .header("Content-Type", "text/html")
            .body(content.into())
            .unwrap_or_else(|_| (StatusCode::NOT_FOUND, "index.html not found").into_response()),
        Err(_) => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}
