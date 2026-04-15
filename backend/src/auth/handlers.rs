use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::PgPool;
use validator::Validate;

use crate::auth::models::{AuthResponse, CreateUserRequest, LoginRequest, UpdateSettingsRequest, UserResponse};
use crate::auth::service::AuthService;
use crate::shared::error::AppResult;

pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<AuthResponse>> {
    req.validate()?;

    let response = AuthService::register(&pool, req).await?;

    Ok(Json(response))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let response = AuthService::login(&pool, req).await?;

    Ok(Json(response))
}

pub async fn verify(
    Json(token): Json<String>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = AuthService::verify_token(&token).await?;

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

/// Extract auth token from either query param or Authorization header.
fn extract_token(query: &SettingsQuery, headers: &axum::http::HeaderMap) -> Result<String, axum::http::StatusCode> {
    // Query param takes priority
    if let Some(ref token) = query.token {
        if !token.is_empty() {
            return Ok(token.clone());
        }
    }

    // Fallback to Authorization: Bearer <token>
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if !token.is_empty() {
                    return Ok(token.to_string());
                }
            }
        }
    }

    Err(axum::http::StatusCode::UNAUTHORIZED)
}

pub async fn get_settings(
    State(pool): State<PgPool>,
    Query(query): Query<SettingsQuery>,
    headers: axum::http::HeaderMap,
) -> AppResult<Json<UserResponse>> {
    let token = extract_token(&query, &headers).map_err(|_| crate::shared::error::AppError::Unauthorized("Missing token".into()))?;
    let claims = AuthService::verify_token(&token).await?;
    let user = AuthService::get_user_by_id(&pool, claims.sub).await?;
    Ok(Json(user.into()))
}

pub async fn update_settings(
    State(pool): State<PgPool>,
    Query(query): Query<SettingsQuery>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateSettingsRequest>,
) -> AppResult<Json<UserResponse>> {
    let token = extract_token(&query, &headers).map_err(|_| crate::shared::error::AppError::Unauthorized("Missing token".into()))?;
    let claims = AuthService::verify_token(&token).await?;
    let user = AuthService::update_settings(&pool, claims.sub, req).await?;
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
    State(pool): State<PgPool>,
    Query(query): Query<VerifyEmailQuery>,
    Json(body): Json<VerifyEmailBody>,
) -> AppResult<Json<UserResponse>> {
    let claims = AuthService::verify_token(&query.token).await?;
    let user = AuthService::verify_email(&pool, claims.sub, body.code).await?;
    Ok(Json(user))
}

pub async fn resend_verification(
    State(pool): State<PgPool>,
    Query(query): Query<VerifyEmailQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = AuthService::verify_token(&query.token).await?;
    let _ = AuthService::resend_verification(&pool, claims.sub).await?;
    Ok(Json(serde_json::json!({"message": "Verification code sent"})))
}