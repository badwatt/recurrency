use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::PgPool;
use validator::Validate;

use crate::auth::models::{AuthResponse, CreateUserRequest, LoginRequest, UpdateSettingsRequest, UserResponse};
use crate::auth::service::AuthService;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;
use crate::shared::OptionalQuery;

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

pub async fn get_settings(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<SettingsQuery>,
    headers: axum::http::HeaderMap,
) -> AppResult<Json<UserResponse>> {
    let token = extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?;
    let claims = AuthService::verify_token(&token).await?;
    let user = AuthService::get_user_by_id(&pool, claims.sub).await?;
    Ok(Json(user.into()))
}

pub async fn update_settings(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<SettingsQuery>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateSettingsRequest>,
) -> AppResult<Json<UserResponse>> {
    let token = extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?;
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