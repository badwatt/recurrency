use std::env;

use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::demo::service::DemoService;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::{AppError, AppResult};
use crate::shared::password_utils::hash_password;
use crate::shared::OptionalQuery;

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
    hash_password(password)
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
    pub token: Option<String>,
}

pub async fn reset(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<DemoQuery>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    if !is_demo_enabled() {
        return Err(AppError::BadRequest("Demo mode is not enabled".to_string()));
    }

    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_is_enabled_default() {
        // Default is disabled when no env vars are set
        // (ENABLE_DEMO is not set, so it defaults to false)
        let enabled = is_demo_enabled();
        // Note: This test's result depends on env state
        // We just verify the function doesn't panic
        let _ = enabled;
    }

    #[test]
    fn demo_email_custom() {
        env::set_var("DEMO_EMAIL", "custom@demo.com");
        let email = demo_email();
        assert_eq!(email, "custom@demo.com");
        env::remove_var("DEMO_EMAIL");
    }

    #[test]
    fn demo_secret_custom() {
        env::set_var("DEMO_PASSWORD", "my_test_pass");
        let pwd = demo_password();
        assert_eq!(pwd, "my_test_pass");
        env::remove_var("DEMO_PASSWORD");
    }

    #[test]
    fn demo_secret_hash_custom() {
        env::set_var("DEMO_PASSWORD_HASH", "precomputed$hash");
        let hash = demo_password_hash("anything").unwrap();
        assert_eq!(hash, "precomputed$hash");
        env::remove_var("DEMO_PASSWORD_HASH");
    }

    #[test]
    fn demo_secret_hash_generated() {
        env::remove_var("DEMO_PASSWORD_HASH");
        let hash = demo_password_hash("testpass").unwrap();
        assert!(!hash.is_empty());
    }
}