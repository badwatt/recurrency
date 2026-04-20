use axum::{
    extract::{Query, State},
    Json,
};
use std::sync::Arc;
use validator::Validate;

use crate::auth::models::{AuthResponse, CreateUserRequest, LoginRequest, SettingsResponse, UpdateSettingsRequest, UserResponse};
use crate::auth::service::AuthServiceTrait;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;
use crate::shared::OptionalQuery;

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

pub async fn verify(
    Json(token): Json<String>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = crate::auth::service::verify_token_internal(&token)?;
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
    let token = extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?;
    let claims = crate::auth::service::verify_token_internal(&token)?;
    let user = state.auth_service.get_settings(claims.sub).await?;
    Ok(Json(user))
}

pub async fn update_settings(
    State(state): State<crate::shared::app_state::AppState>,
    OptionalQuery(query): OptionalQuery<SettingsQuery>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateSettingsRequest>,
) -> AppResult<Json<SettingsResponse>> {
    let token = extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?;
    let claims = crate::auth::service::verify_token_internal(&token)?;
    let user = state.auth_service.update_settings(claims.sub, req).await?;
    Ok(Json(user.into()))
}

#[derive(serde::Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(serde::Deserialize)]
pub struct VerifyEmailBody {
    pub code: String,
}

pub async fn verify_email(
    State(state): State<crate::shared::app_state::AppState>,
    Query(query): Query<VerifyEmailQuery>,
    Json(body): Json<VerifyEmailBody>,
) -> AppResult<Json<UserResponse>> {
    let claims = crate::auth::service::verify_token_internal(&query.token)?;
    let user = state.auth_service.verify_email(claims.sub, body.code).await?;
    Ok(Json(user))
}

pub async fn resend_verification(
    State(state): State<crate::shared::app_state::AppState>,
    Query(query): Query<VerifyEmailQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = crate::auth::service::verify_token_internal(&query.token)?;
    let _ = state.auth_service.resend_verification(claims.sub).await?;
    Ok(Json(serde_json::json!({"message": "Verification code sent"})))
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
        assert_eq!(result.token, "mytoken123");
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
        assert_eq!(response.get("email").unwrap().as_str().unwrap(), "test@example.com");
    }

    #[test]
    fn resend_verification_response_structure() {
        let response = serde_json::json!({"message": "Verification code sent"});
        assert_eq!(response.get("message").unwrap().as_str().unwrap(), "Verification code sent");
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
}
