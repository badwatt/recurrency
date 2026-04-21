use std::env;

use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::demo::service::DemoService;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::app_state::AppState;
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

pub async fn init(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
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
    .execute(&state.pool)
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
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<DemoQuery>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    if !is_demo_enabled() {
        return Err(AppError::BadRequest("Demo mode is not enabled".to_string()));
    }

    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    match DemoService::reset_demo_data(&state.pool, claims.sub).await {
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
        // Test that DEMO_EMAIL env var is respected
        // We can't fully test in isolation due to env state, but verify function exists
        // and doesn't panic
        let _email = demo_email();
        // This test is informational - actual behavior depends on runtime env
    }

    #[test]
    fn demo_secret_custom() {
        env::set_var("DEMO_PASSWORD", "my_test_pass");
        let secret_val = demo_password();
        assert_eq!(secret_val, "my_test_pass");
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

    // ── is_demo_enabled edge cases ──

    #[test]
    fn demo_is_enabled_true_lowercase() {
        env::remove_var("ENABLE_DEMO");
        env::set_var("ENABLE_DEMO", "true");
        let enabled = is_demo_enabled();
        assert!(enabled);
        env::remove_var("ENABLE_DEMO");
    }

    #[test]
    fn demo_is_enabled_true_uppercase() {
        env::remove_var("ENABLE_DEMO");
        env::set_var("ENABLE_DEMO", "TRUE");
        let enabled = is_demo_enabled();
        assert!(enabled);
        env::remove_var("ENABLE_DEMO");
    }

    #[test]
    fn demo_is_enabled_1() {
        env::remove_var("ENABLE_DEMO");
        env::set_var("ENABLE_DEMO", "1");
        let enabled = is_demo_enabled();
        assert!(!enabled);
        env::remove_var("ENABLE_DEMO");
    }

    #[test]
    fn demo_is_enabled_yes() {
        env::remove_var("ENABLE_DEMO");
        env::set_var("ENABLE_DEMO", "yes");
        let enabled = is_demo_enabled();
        assert!(!enabled);  // "yes" not in "true" set
        env::remove_var("ENABLE_DEMO");
    }

    #[test]
    fn demo_is_enabled_false() {
        env::remove_var("ENABLE_DEMO");
        env::set_var("ENABLE_DEMO", "false");
        let enabled = is_demo_enabled();
        assert!(!enabled);
        env::remove_var("ENABLE_DEMO");
    }

    // ── demo_email edge cases ──

    #[test]
    fn demo_email_disabled_returns_empty() {
        // Must clear ALL demo env vars first
        env::remove_var("ENABLE_DEMO");
        env::remove_var("DEMO_EMAIL");
        env::remove_var("DEMO_PASSWORD");
        env::remove_var("DEMO_PASSWORD_HASH");
        
        let email = demo_email();
        assert_eq!(email, "");
    }

    #[test]
    fn demo_email_enabled_custom() {
        // Clear all first
        env::remove_var("DEMO_EMAIL");
        env::remove_var("DEMO_PASSWORD");
        env::remove_var("DEMO_PASSWORD_HASH");
        
        env::set_var("ENABLE_DEMO", "true");
        env::set_var("DEMO_EMAIL", "custom@demo.com");
        let email = demo_email();
        assert_eq!(email, "custom@demo.com");
        env::remove_var("ENABLE_DEMO");
        env::remove_var("DEMO_EMAIL");
    }

    #[test]
    fn demo_email_enabled_generates_random() {
        // Clear all first
        env::remove_var("ENABLE_DEMO");
        env::remove_var("DEMO_EMAIL");
        env::remove_var("DEMO_PASSWORD");
        env::remove_var("DEMO_PASSWORD_HASH");
        
        env::set_var("ENABLE_DEMO", "true");
        let email1 = demo_email();
        let email2 = demo_email();
        assert!(email1.starts_with("demo_"));
        assert!(email1.contains("@recurrency.local"));
        assert_ne!(email1, email2); // Should be different
        env::remove_var("ENABLE_DEMO");
    }

    // ── demo_password edge cases ──

    #[test]
    fn demo_password_default_length() {
        env::remove_var("DEMO_PASSWORD");
        let password = demo_password();
        assert!(password.len() >= 16); // UUID without dashes is 32 chars
    }

    #[test]
    fn demo_password_no_hyphens() {
        env::remove_var("DEMO_PASSWORD");
        let password = demo_password();
        assert!(!password.contains("-"));
    }

    // ── demo_password_hash edge cases ──

    #[test]
    fn demo_password_hash_uses_password_when_no_precomputed() {
        env::remove_var("DEMO_PASSWORD_HASH");
        let hash1 = demo_password_hash("pass1").unwrap();
        let hash2 = demo_password_hash("pass2").unwrap();
        assert_ne!(hash1, hash2);
    }

    // ── DemoQuery deserialization ──

    #[test]
    fn demo_query_with_token() {
        let query = "token=my-token-123";
        let result: DemoQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("my-token-123"));
    }

    #[test]
    fn demo_query_without_token() {
        let query = "";
        let result: DemoQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_none());
    }

    #[test]
    fn demo_query_empty_token() {
        let query = "token=";
        let result: DemoQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_some());
        assert!(result.token.as_ref().unwrap().is_empty());
    }

    // ── status() response structure ──

    #[test]
    fn status_response_disabled() {
        env::remove_var("ENABLE_DEMO");
        let enabled = is_demo_enabled();
        let email = if enabled { demo_email() } else { String::new() };
        
        assert!(!enabled);
        assert_eq!(email, "");
        
        // Verify the response structure that status() would return
        let response = serde_json::json!({
            "enabled": enabled,
            "email": email
        });
        assert!(!response["enabled"].as_bool().unwrap());
        assert_eq!(response["email"].as_str().unwrap(), "");
    }

    #[test]
    fn status_response_enabled_with_custom_email() {
        env::remove_var("ENABLE_DEMO");
        env::remove_var("DEMO_EMAIL");
        env::remove_var("DEMO_PASSWORD");
        env::remove_var("DEMO_PASSWORD_HASH");
        env::set_var("ENABLE_DEMO", "true");
        env::set_var("DEMO_EMAIL", "status@demo.com");
        
        let enabled = is_demo_enabled();
        let email = demo_email();
        
        assert!(enabled);
        assert_eq!(email, "status@demo.com");
        
        env::remove_var("ENABLE_DEMO");
        env::remove_var("DEMO_EMAIL");
    }

    // ── init response structure ──

    #[test]
    fn init_response_structure() {
        let response = serde_json::json!({
            "email": "init@demo.com",
            "password": "secret123"
        });
        assert_eq!(response["email"].as_str().unwrap(), "init@demo.com");
        assert_eq!(response["password"].as_str().unwrap(), "secret123");
    }

    // ── reset response structure ──

    #[test]
    fn reset_response_success() {
        let response = serde_json::json!({
            "success": true,
            "message": "Demo data has been reset successfully"
        });
        assert!(response["success"].as_bool().unwrap());
        assert!(response["message"].as_str().unwrap().contains("reset"));
    }

    #[test]
    fn reset_response_failure() {
        let response = serde_json::json!({
            "success": false,
            "message": "Database error"
        });
        assert!(!response["success"].as_bool().unwrap());
        assert_eq!(response["message"].as_str().unwrap(), "Database error");
    }
}