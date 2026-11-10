use axum::{Json, extract::State};
use validator::Validate;

use crate::auth::models::{
    AuthResponse, CreateUserRequest, DeleteAccountRequest, LoginRequest, SettingsResponse,
    UpdateSettingsRequest, UserResponse, is_valid_email,
};
use crate::shared::OptionalQuery;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;

pub async fn register(
    State(state): State<crate::shared::app_state::AppState>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<AuthResponse>> {
    req.validate()?;
    let response = state.auth_service.register(req).await?;
    Ok(Json(response))
}

pub async fn login(
    State(state): State<crate::shared::app_state::AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let response = state.auth_service.login(req).await?;
    Ok(Json(response))
}

/// GET /api/auth/verify - uses static JWT token (no State needed)
pub async fn verify(Json(token): Json<String>) -> AppResult<Json<serde_json::Value>> {
    let claims = crate::auth::service::verify_token_internal(&token)?;
    Ok(Json(serde_json::json!({
        "valid": true,
        "user_id": claims.sub,
        "email": claims.email
    })))
}

/// POST /api/auth/verify - uses mock auth service for testing
pub async fn verify_post(
    State(state): State<crate::shared::app_state::AppState>,
    Json(token): Json<String>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = state.auth_service.verify_token(&token).await?;
    Ok(Json(serde_json::json!({
        "valid": true,
        "user_id": claims.sub,
        "email": claims.email
    })))
}

#[derive(serde::Deserialize)]
pub struct SettingsQuery {
    pub token: Option<String>,
}

pub async fn get_settings(
    State(state): State<crate::shared::app_state::AppState>,
    OptionalQuery(query): OptionalQuery<SettingsQuery>,
    headers: axum::http::HeaderMap,
) -> AppResult<Json<UserResponse>> {
    let token =
        extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?;
    let claims = state.auth_service.verify_token(&token).await?;
    let user = state.auth_service.get_settings(claims.sub).await?;
    Ok(Json(user))
}

pub async fn update_settings(
    State(state): State<crate::shared::app_state::AppState>,
    OptionalQuery(query): OptionalQuery<SettingsQuery>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateSettingsRequest>,
) -> AppResult<Json<SettingsResponse>> {
    let token =
        extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?;
    let claims = state.auth_service.verify_token(&token).await?;
    if let Some(email) = req.email.as_deref()
        && !is_valid_email(email)
    {
        return Err(crate::shared::error::AppError::BadRequest(
            "Invalid email".to_string(),
        ));
    }
    let user = state.auth_service.update_settings(claims.sub, req).await?;
    Ok(Json(user.into()))
}

/// DELETE /api/settings/account - delete the authenticated account after password confirmation.
pub async fn delete_account(
    State(state): State<crate::shared::app_state::AppState>,
    OptionalQuery(query): OptionalQuery<SettingsQuery>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DeleteAccountRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let token =
        extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?;
    let claims = state.auth_service.verify_token(&token).await?;
    state
        .auth_service
        .delete_account(claims.sub, req.password)
        .await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

#[derive(serde::Deserialize)]
pub struct VerifyEmailQuery {
    pub token: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct VerifyEmailBody {
    pub code: String,
}

pub async fn verify_email(
    State(state): State<crate::shared::app_state::AppState>,
    OptionalQuery(query): OptionalQuery<VerifyEmailQuery>,
    headers: axum::http::HeaderMap,
    Json(body): Json<VerifyEmailBody>,
) -> AppResult<Json<UserResponse>> {
    let token =
        extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?;
    let claims = state.auth_service.verify_verification_token(&token).await?;
    let user = state
        .auth_service
        .verify_email(claims.sub, body.code)
        .await?;
    Ok(Json(user))
}

pub async fn resend_verification(
    State(state): State<crate::shared::app_state::AppState>,
    OptionalQuery(query): OptionalQuery<VerifyEmailQuery>,
    headers: axum::http::HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let token =
        extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?;
    let claims = state.auth_service.verify_verification_token(&token).await?;
    let _ = state.auth_service.resend_verification(claims.sub).await?;
    Ok(Json(
        serde_json::json!({"message": "Verification code sent"}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::models::{AuthResponse, Claims, UserResponse};
    use validator::Validate;

    fn make_create_request(email: &str, secret_val: &str, name: &str) -> CreateUserRequest {
        let json = format!(
            r#"{{"email":"{}","password":"{}","name":"{}"}}"#,
            email, secret_val, name
        );
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn verify_email_query_deserialize() {
        let query = "token=mytoken123";
        let result: VerifyEmailQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token, Some("mytoken123".to_string()));
    }

    #[test]
    fn verify_email_body_deserialize() {
        let json = r#"{"code":"123456"}"#;
        let body: VerifyEmailBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.code, "123456");
    }

    #[test]
    fn settings_query_with_token() {
        let query = "token=my-token";
        let result: SettingsQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("my-token"));
    }

    #[test]
    fn settings_query_without_token() {
        let query = "";
        let result: SettingsQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_none());
    }

    #[test]
    fn create_user_request_validation() {
        let req = make_create_request("valid@example.com", "valid_pwd_val", "Test");
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_user_request_validation_bad_email() {
        let req = make_create_request("invalid", "valid_pwd_val", "Test");
        assert!(req.validate().is_err());
    }

    #[test]
    fn login_request_deserialize() {
        let json = r#"{"email":"test@test.com","password":"test_secret_val"}"#;
        let req: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "test@test.com");
        assert_eq!(req.password, "test_secret_val");
    }

    #[test]
    fn update_settings_request_deserialize() {
        let json = r#"{"name":"New Name","currency":"GBP"}"#;
        let req: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("New Name"));
        assert_eq!(req.currency.as_deref(), Some("GBP"));
    }

    #[test]
    fn verify_response_structure() {
        let response = serde_json::json!({
            "valid": true,
            "user_id": "123e4567-e89b-12d3-a456-426614174000",
            "email": "test@example.com"
        });
        assert!(response.get("valid").unwrap().as_bool().unwrap());
        assert!(response.get("user_id").is_some());
        assert_eq!(
            response.get("email").unwrap().as_str().unwrap(),
            "test@example.com"
        );
    }

    #[test]
    fn resend_verification_response_structure() {
        let response = serde_json::json!({"message": "Verification code sent"});
        assert_eq!(
            response.get("message").unwrap().as_str().unwrap(),
            "Verification code sent"
        );
    }

    #[test]
    fn register_validation_fails_short_password() {
        let req = make_create_request("test@example.com", "short", "Test");
        assert!(req.validate().is_err());
    }

    #[test]
    fn register_validation_fails_empty_email() {
        let req = make_create_request("", "validpassword123", "Test");
        assert!(req.validate().is_err());
    }

    #[test]
    fn settings_query_empty_string_token() {
        let query = "token=";
        let result: SettingsQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_some());
        assert!(result.token.as_ref().unwrap().is_empty());
    }

    #[test]
    fn update_settings_empty_body() {
        let json = r#"{}"#;
        let req: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.currency.is_none());
    }

    #[test]
    fn user_response_serialization() {
        let user = UserResponse {
            id: uuid::Uuid::new_v4(),
            email: "user@test.com".to_string(),
            avatar: None,
            name: None,
            currency: "EUR".to_string(),
            verified: false,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("user@test.com"));
        assert!(json.contains("EUR"));
    }

    #[test]
    fn claims_structure() {
        let claims = Claims {
            sub: uuid::Uuid::new_v4(),
            email: "claims@test.com".to_string(),
            exp: 9999999999,
            iat: 1000000000,
        };
        assert_eq!(claims.email, "claims@test.com");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn auth_response_serialization() {
        let user = UserResponse {
            id: uuid::Uuid::new_v4(),
            email: "auth@test.com".to_string(),
            avatar: None,
            name: Some("Test User".to_string()),
            currency: "USD".to_string(),
            verified: true,
        };
        let resp = AuthResponse {
            token: "jwt-token".to_string(),
            user,
            requires_verification: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("jwt-token"));
        assert!(json.contains("auth@test.com"));
    }

    #[test]
    fn verify_email_body_missing_code() {
        let json = r#"{}"#;
        let result: Result<VerifyEmailBody, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn verify_email_query_deserialize_with_special_chars() {
        // serde_urlencoded decodes %40 to @ automatically
        let query = "token=abc%40test.com";
        let result: VerifyEmailQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token, Some("abc@test.com".to_string()));
    }
}

// ── Handler Integration Tests ─────────────────────────────────────────────────
// These require a running PostgreSQL database. They skip gracefully if unavailable.

#[cfg(test)]
mod handler_integration_tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::{get, post, put};
    use tower::ServiceExt;

    // Valid mock token for MockAuthService (starts with mock_token_ + valid UUID)
    const VALID_MOCK_TOKEN: &str = "mock_token_00000000-0000-0000-0000-000000000001";

    async fn make_app() -> Option<Router> {
        let state = crate::shared::app_state::create_test_app_state().await?;
        Some(
            Router::new()
                .route("/api/auth/register", post(register))
                .route("/api/auth/login", post(login))
                .route("/api/auth/verify", post(verify_post))
                .route("/api/auth/verify-email", post(verify_email))
                .route("/api/auth/resend-verification", post(resend_verification))
                .route("/api/settings", get(get_settings))
                .route("/api/settings", put(update_settings))
                .route("/api/settings/account", post(delete_account))
                .with_state(state),
        )
    }

    // ── Register ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn handler_register_success() {
        let Some(app) = make_app().await else {
            return;
        };
        let body = serde_json::json!({
            "email": "new@test.com",
            "password": "password123",
            "name": "New User"
        });
        let (status, _body) =
            post_json::<_, serde_json::Value>(&app, "/api/auth/register", &body).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn handler_register_validation_empty_email() {
        let Some(app) = make_app().await else {
            return;
        };
        // Axum returns 400 for validation errors from the validator derive
        let body = serde_json::json!({"email": "", "password": "short", "name": "Test"});
        let (status, _body) =
            post_json::<_, serde_json::Value>(&app, "/api/auth/register", &body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn handler_register_invalid_email_format() {
        let Some(app) = make_app().await else {
            return;
        };
        let body =
            serde_json::json!({"email": "not-an-email", "password": "password123", "name": "Test"});
        let (status, _body) =
            post_json::<_, serde_json::Value>(&app, "/api/auth/register", &body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn handler_register_duplicate_email() {
        let Some(app) = make_app().await else {
            return;
        };
        let body = serde_json::json!({"email": "dup@test.com", "password": "password123", "name": "First"});
        post_json::<_, serde_json::Value>(&app, "/api/auth/register", &body).await;
        let (status, _body) =
            post_json::<_, serde_json::Value>(&app, "/api/auth/register", &body).await;
        assert_eq!(status, StatusCode::CONFLICT);
    }

    // ── Login ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn handler_login_success() {
        let Some(app) = make_app().await else {
            return;
        };
        let reg = serde_json::json!({"email": "login@test.com", "password": "password123", "name": "Login User"});
        post_json::<_, serde_json::Value>(&app, "/api/auth/register", &reg).await;
        let login = serde_json::json!({"email": "login@test.com", "password": "password123"});
        let (status, _body) =
            post_json::<_, serde_json::Value>(&app, "/api/auth/login", &login).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn handler_login_unknown_user() {
        let Some(app) = make_app().await else {
            return;
        };
        let body = serde_json::json!({"email": "nobody@test.com", "password": "password123"});
        let (status, _body) =
            post_json::<_, serde_json::Value>(&app, "/api/auth/login", &body).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    // ── Verify (POST - uses mock) ────────────────────────────────────────

    #[tokio::test]
    async fn handler_verify_valid_mock_token() {
        let Some(app) = make_app().await else {
            return;
        };
        let token_str = VALID_MOCK_TOKEN.to_string();
        let body = serde_json::to_string(&token_str).unwrap();
        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/verify")
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn handler_verify_invalid_mock_token() {
        let Some(app) = make_app().await else {
            return;
        };
        let token_str = "not-a-valid-mock-token".to_string();
        let body = serde_json::to_string(&token_str).unwrap();
        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/verify")
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ── Get Settings ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn handler_get_settings_success() {
        let Some(app) = make_app().await else {
            return;
        };
        // Register user and get the mock token from the response
        let reg = serde_json::json!({"email": "settings@test.com", "password": "password123", "name": "Settings"});
        let (_, resp): (_, serde_json::Value) = post_json(&app, "/api/auth/register", &reg).await;
        let token = resp["token"].as_str().unwrap();
        // Now call get_settings using Authorization header
        let req = Request::builder()
            .method("GET")
            .uri("/api/settings")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        eprintln!(
            "get_settings: status={} body={:?}",
            status,
            String::from_utf8_lossy(&body)
        );
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn handler_get_settings_invalid_token() {
        let Some(app) = make_app().await else {
            return;
        };
        let req = Request::builder()
            .method("GET")
            .uri("/api/settings?token=bad_token")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn handler_get_settings_no_token() {
        let Some(app) = make_app().await else {
            return;
        };
        let req = Request::builder()
            .method("GET")
            .uri("/api/settings")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ── Update Settings ─────────────────────────────────────────────────

    #[tokio::test]
    async fn handler_update_settings_success() {
        let Some(app) = make_app().await else {
            return;
        };
        // Register user and get the mock token from the response
        let reg = serde_json::json!({"email": "update@test.com", "password": "password123", "name": "Update"});
        let (_, resp): (_, serde_json::Value) = post_json(&app, "/api/auth/register", &reg).await;
        let token = resp["token"].as_str().unwrap();
        // Now call update_settings using Authorization header
        let body = serde_json::json!({"name": "Updated Name", "currency": "GBP"});
        let req = Request::builder()
            .method("PUT")
            .uri("/api/settings")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn handler_update_settings_invalid_token() {
        let Some(app) = make_app().await else {
            return;
        };
        let body = serde_json::json!({"name": "New Name"});
        let (status, _body) =
            put_json::<_, serde_json::Value>(&app, "/api/settings?token=invalid", &body).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    // ── Delete Account ───────────────────────────────────────────────────

    #[tokio::test]
    async fn handler_delete_account_success() {
        let Some(app) = make_app().await else {
            return;
        };
        let reg = serde_json::json!({"email": "delete@test.com", "password": "password123", "name": "Delete"});
        let (_, resp): (_, serde_json::Value) = post_json(&app, "/api/auth/register", &reg).await;
        let token = resp["token"].as_str().unwrap();
        let body = serde_json::json!({"password": "password123"});
        let req = Request::builder()
            .method("POST")
            .uri("/api/settings/account")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn handler_delete_account_wrong_password() {
        let Some(app) = make_app().await else {
            return;
        };
        let reg = serde_json::json!({"email": "delete-wrong@test.com", "password": "password123", "name": "Delete"});
        let (_, resp): (_, serde_json::Value) = post_json(&app, "/api/auth/register", &reg).await;
        let token = resp["token"].as_str().unwrap();
        let body = serde_json::json!({"password": "wrong-password"});
        let req = Request::builder()
            .method("POST")
            .uri("/api/settings/account")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ── Verify Email ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn handler_verify_email_success() {
        let Some(app) = make_app().await else {
            return;
        };
        // Register user and get the mock token from the response
        let reg = serde_json::json!({"email": "verify@test.com", "password": "password123", "name": "Verify"});
        let (_, resp): (_, serde_json::Value) = post_json(&app, "/api/auth/register", &reg).await;
        let token = resp["token"].as_str().unwrap();
        // Now call verify_email with the registered user's token
        let uri = format!("/api/auth/verify-email?token={}", token);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"code":"123456"}"#))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        let status = response.status();
        if status != StatusCode::OK {
            let body = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            eprintln!(
                "verify_email: status={} body={:?}",
                status,
                String::from_utf8_lossy(&body)
            );
        }
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn handler_verify_email_invalid_token() {
        let Some(app) = make_app().await else {
            return;
        };
        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/verify-email?token=invalid")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"code":"123456"}"#))
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ── Resend Verification ─────────────────────────────────────────────

    #[tokio::test]
    async fn handler_resend_verification_success() {
        let Some(app) = make_app().await else {
            return;
        };
        // Register user and get the mock token from the response
        let reg = serde_json::json!({"email": "resend@test.com", "password": "password123", "name": "Resend"});
        let (_, resp): (_, serde_json::Value) = post_json(&app, "/api/auth/register", &reg).await;
        let token = resp["token"].as_str().unwrap();
        // Now call resend_verification
        let uri = format!("/api/auth/resend-verification?token={}", token);
        let req = Request::builder()
            .method("POST")
            .uri(&uri)
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn handler_resend_verification_invalid_token() {
        let Some(app) = make_app().await else {
            return;
        };
        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/resend-verification?token=invalid")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    async fn post_json<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        app: &Router,
        uri: &str,
        body: &T,
    ) -> (StatusCode, R) {
        let json_body = serde_json::to_string(body).unwrap();
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(json_body))
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body: R = serde_json::from_slice(&body_bytes).unwrap_or_else(|_| {
            panic!(
                "Failed to parse response: {:?}",
                String::from_utf8_lossy(&body_bytes)
            )
        });
        (status, body)
    }

    async fn put_json<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        app: &Router,
        uri: &str,
        body: &T,
    ) -> (StatusCode, R) {
        let json_body = serde_json::to_string(body).unwrap();
        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(json_body))
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body: R = serde_json::from_slice(&body_bytes).unwrap_or_else(|_| {
            panic!(
                "Failed to parse response: {:?}",
                String::from_utf8_lossy(&body_bytes)
            )
        });
        (status, body)
    }
}
