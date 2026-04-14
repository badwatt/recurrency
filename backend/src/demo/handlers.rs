use std::env;

use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};
use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::demo::service::DemoService;
use crate::shared::error::{AppError, AppResult};

/// Check if demo mode is enabled via environment variable.
fn is_demo_enabled() -> bool {
    env::var("ENABLE_DEMO")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Get demo email from env or generate random one.
fn demo_email() -> String {
    env::var("DEMO_EMAIL").unwrap_or_else(|_| {
        if is_demo_enabled() {
            // Generate random email when enabled but not configured
            format!("demo_{}@recurrency.local", &Uuid::new_v4().to_string()[..8])
        } else {
            String::new()
        }
    })
}

/// Get demo password from env or generate random one.
fn demo_password() -> String {
    env::var("DEMO_PASSWORD")
        .unwrap_or_else(|_| Uuid::new_v4().to_string().replace("-", "")[..16].to_string())
}

/// Get demo password hash. Only needed when creating the user.
fn demo_password_hash(password: &str) -> Result<String, AppError> {
    if let Ok(hash) = env::var("DEMO_PASSWORD_HASH") {
        return Ok(hash);
    }

    // Generate hash from the given password
    argon2_hash(password)
}

fn argon2_hash(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))
}

/// Get demo status for frontend.
pub async fn status() -> AppResult<Json<serde_json::Value>> {
    let enabled = is_demo_enabled();
    let email = if enabled { demo_email() } else { String::new() };

    Ok(Json(serde_json::json!({
        "enabled": enabled,
        "email": email
    })))
}

pub async fn init(State(pool): State<PgPool>) -> AppResult<Json<serde_json::Value>> {
    if !is_demo_enabled() {
        return Err(AppError::BadRequest("Demo mode is not enabled".to_string()));
    }

    let email = demo_email();
    let password = demo_password();
    let hash = demo_password_hash(&password)?;

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, name, currency, verified, created_at, updated_at)
        VALUES ($1, $2, $3, 'Demo User', 'EUR', true, NOW(), NOW())
        ON CONFLICT (email) DO UPDATE SET
            password_hash = EXCLUDED.password_hash,
            verified = true,
            name = EXCLUDED.name,
            updated_at = NOW()
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&email)
    .bind(&hash)
    .execute(&pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to init demo user: {}", e)))?;

    Ok(Json(serde_json::json!({
        "email": email,
        "password": password
    })))
}

#[derive(serde::Deserialize)]
pub struct DemoQuery {
    pub token: String,
}

pub async fn reset(
    State(pool): State<PgPool>,
    Query(query): Query<DemoQuery>,
) -> AppResult<Json<serde_json::Value>> {
    if !is_demo_enabled() {
        return Err(AppError::BadRequest("Demo mode is not enabled".to_string()));
    }

    let claims = AuthService::verify_token(&query.token).await?;

    match DemoService::reset_demo_data(&pool, claims.sub).await {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Demo data has been reset successfully"
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "message": e
        }))),
    }
}

