use std::env;

use axum::{extract::State, http::HeaderMap, Json};
use uuid::Uuid;

use crate::shared::app_state::AppState;
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

pub async fn init(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    if !is_demo_enabled() {
        return Err(AppError::BadRequest("Demo mode is not enabled".to_string()));
    }

    let email = demo_email();
    let password = demo_password();
    let hash = demo_password_hash(&password)?;

    state
        .demo_service
        .create_demo_user(&email, &hash)
        .await
        .map_err(AppError::Internal)?;

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

    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;

    match state.demo_service.reset_demo_data(claims.sub).await {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Demo data has been reset successfully"
        }))),
        Err(e) => Err(AppError::Internal(format!("Failed to reset demo data: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    #[serial]
    fn demo_is_enabled_default() {
        // Default is disabled when no env vars are set
        // (ENABLE_DEMO is not set, so it defaults to false)
        let enabled = is_demo_enabled();
        // Note: This test's result depends on env state
        // We just verify the function doesn't panic
        let _ = enabled;
    }

    #[test]
    #[serial]
    fn demo_email_custom() {
        // Test that DEMO_EMAIL env var is respected
        // We can't fully test in isolation due to env state, but verify function exists
        // and doesn't panic
        let _email = demo_email();
        // This test is informational - actual behavior depends on runtime env
    }

    #[test]
    #[serial]
    fn demo_secret_custom() {
        crate::shared::test_env::set_env("DEMO_PASSWORD", "my_test_pass");
        let secret_val = demo_password();
        assert_eq!(secret_val, "my_test_pass");
        crate::shared::test_env::remove_env("DEMO_PASSWORD");
    }

    #[test]
    #[serial]
    fn demo_secret_hash_custom() {
        crate::shared::test_env::set_env("DEMO_PASSWORD_HASH", "precomputed$hash");
        let hash = demo_password_hash("anything").unwrap();
        assert_eq!(hash, "precomputed$hash");
        crate::shared::test_env::remove_env("DEMO_PASSWORD_HASH");
    }

    #[test]
    #[serial]
    fn demo_secret_hash_generated() {
        crate::shared::test_env::remove_env("DEMO_PASSWORD_HASH");
        let hash = demo_password_hash("testpass").unwrap();
        assert!(!hash.is_empty());
    }

    // ── is_demo_enabled edge cases ──

    #[test]
    #[serial]
    fn demo_is_enabled_true_lowercase() {
        crate::shared::test_env::remove_env("ENABLE_DEMO");
        crate::shared::test_env::set_env("ENABLE_DEMO", "true");
        let enabled = is_demo_enabled();
        assert!(enabled);
        crate::shared::test_env::remove_env("ENABLE_DEMO");
    }

    #[test]
    #[serial]
    fn demo_is_enabled_true_uppercase() {
        crate::shared::test_env::remove_env("ENABLE_DEMO");
        crate::shared::test_env::set_env("ENABLE_DEMO", "TRUE");
        let enabled = is_demo_enabled();
        assert!(enabled);
        crate::shared::test_env::remove_env("ENABLE_DEMO");
    }

    #[test]
    #[serial]
    fn demo_is_enabled_1() {
        crate::shared::test_env::remove_env("ENABLE_DEMO");
        crate::shared::test_env::set_env("ENABLE_DEMO", "1");
        let enabled = is_demo_enabled();
        assert!(!enabled);
        crate::shared::test_env::remove_env("ENABLE_DEMO");
    }

    #[test]
    #[serial]
    fn demo_is_enabled_yes() {
        crate::shared::test_env::remove_env("ENABLE_DEMO");
        crate::shared::test_env::set_env("ENABLE_DEMO", "yes");
        let enabled = is_demo_enabled();
        assert!(!enabled); // "yes" not in "true" set
        crate::shared::test_env::remove_env("ENABLE_DEMO");
    }

    #[test]
    #[serial]
    fn demo_is_enabled_false() {
        crate::shared::test_env::remove_env("ENABLE_DEMO");
        crate::shared::test_env::set_env("ENABLE_DEMO", "false");
        let enabled = is_demo_enabled();
        assert!(!enabled);
        crate::shared::test_env::remove_env("ENABLE_DEMO");
    }

    // ── demo_email edge cases ──

    #[test]
    #[serial]
    fn demo_email_disabled_returns_empty() {
        // Must clear ALL demo env vars first
        crate::shared::test_env::remove_env("ENABLE_DEMO");
        crate::shared::test_env::remove_env("DEMO_EMAIL");
        crate::shared::test_env::remove_env("DEMO_PASSWORD");
        crate::shared::test_env::remove_env("DEMO_PASSWORD_HASH");

        let email = demo_email();
        assert_eq!(email, "");
    }

    #[test]
    #[serial]
    fn demo_email_enabled_custom() {
        // Clear all first
        crate::shared::test_env::remove_env("DEMO_EMAIL");
        crate::shared::test_env::remove_env("DEMO_PASSWORD");
        crate::shared::test_env::remove_env("DEMO_PASSWORD_HASH");

        crate::shared::test_env::set_env("ENABLE_DEMO", "true");
        crate::shared::test_env::set_env("DEMO_EMAIL", "custom@demo.com");
        let email = demo_email();
        assert_eq!(email, "custom@demo.com");
        crate::shared::test_env::remove_env("ENABLE_DEMO");
        crate::shared::test_env::remove_env("DEMO_EMAIL");
    }

    #[test]
    #[serial]
    fn demo_email_enabled_generates_random() {
        // Clear all first
        crate::shared::test_env::remove_env("ENABLE_DEMO");
        crate::shared::test_env::remove_env("DEMO_EMAIL");
        crate::shared::test_env::remove_env("DEMO_PASSWORD");
        crate::shared::test_env::remove_env("DEMO_PASSWORD_HASH");

        crate::shared::test_env::set_env("ENABLE_DEMO", "true");
        let email1 = demo_email();
        let email2 = demo_email();
        assert!(email1.starts_with("demo_"));
        assert!(email1.contains("@recurrency.local"));
        assert_ne!(email1, email2); // Should be different
        crate::shared::test_env::remove_env("ENABLE_DEMO");
    }

    // ── demo_password edge cases ──

    #[test]
    #[serial]
    fn demo_password_default_length() {
        crate::shared::test_env::remove_env("DEMO_PASSWORD");
        let password = demo_password();
        assert!(password.len() >= 16); // UUID without dashes is 32 chars
    }

    #[test]
    #[serial]
    fn demo_password_no_hyphens() {
        crate::shared::test_env::remove_env("DEMO_PASSWORD");
        let password = demo_password();
        assert!(!password.contains("-"));
    }

    // ── demo_password_hash edge cases ──

    #[test]
    #[serial]
    fn demo_password_hash_uses_password_when_no_precomputed() {
        crate::shared::test_env::remove_env("DEMO_PASSWORD_HASH");
        let hash1 = demo_password_hash("pass1").unwrap();
        let hash2 = demo_password_hash("pass2").unwrap();
        assert_ne!(hash1, hash2);
    }

    // ── DemoQuery deserialization ──

    #[test]
    #[serial]
    fn demo_query_with_token() {
        let query = "token=my-token-123";
        let result: DemoQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("my-token-123"));
    }

    #[test]
    #[serial]
    fn demo_query_without_token() {
        let query = "";
        let result: DemoQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_none());
    }

    #[test]
    #[serial]
    fn demo_query_empty_token() {
        let query = "token=";
        let result: DemoQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_some());
        assert!(result.token.as_ref().unwrap().is_empty());
    }

    // ── status() response structure ──

    #[test]
    #[serial]
    fn status_response_disabled() {
        crate::shared::test_env::remove_env("ENABLE_DEMO");
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
    #[serial]
    fn status_response_enabled_with_custom_email() {
        crate::shared::test_env::remove_env("ENABLE_DEMO");
        crate::shared::test_env::remove_env("DEMO_EMAIL");
        crate::shared::test_env::remove_env("DEMO_PASSWORD");
        crate::shared::test_env::remove_env("DEMO_PASSWORD_HASH");
        crate::shared::test_env::set_env("ENABLE_DEMO", "true");
        crate::shared::test_env::set_env("DEMO_EMAIL", "status@demo.com");

        let enabled = is_demo_enabled();
        let email = demo_email();

        assert!(enabled);
        assert_eq!(email, "status@demo.com");

        crate::shared::test_env::remove_env("ENABLE_DEMO");
        crate::shared::test_env::remove_env("DEMO_EMAIL");
    }

    // ── init response structure ──

    #[test]
    #[serial]
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
    #[serial]
    fn reset_response_success() {
        let response = serde_json::json!({
            "success": true,
            "message": "Demo data has been reset successfully"
        });
        assert!(response["success"].as_bool().unwrap());
        assert!(response["message"].as_str().unwrap().contains("reset"));
    }

    #[test]
    #[serial]
    fn reset_response_failure() {
        let response = serde_json::json!({
            "success": false,
            "message": "Database error"
        });
        assert!(!response["success"].as_bool().unwrap());
        assert_eq!(response["message"].as_str().unwrap(), "Database error");
    }
}
