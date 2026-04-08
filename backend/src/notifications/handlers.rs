use axum::{
    extract::{Path, Query, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::notifications::models::{
    NotificationPreferencesResponse, NotificationResponse,
    UpdateNotificationPreferencesRequest,
};
use crate::notifications::service::NotificationService;
use crate::shared::error::{AppError, AppResult};

#[derive(serde::Deserialize)]
pub struct NotificationQuery {
    pub token: String,
}

pub async fn list(
    State(pool): State<PgPool>,
    Query(query): Query<NotificationQuery>,
) -> AppResult<Json<NotificationResponse>> {
    let claims = AuthService::verify_token(&query.token).await?;
    let notifications = NotificationService::list(&pool, claims.sub).await;
    Ok(Json(notifications))
}

pub async fn mark_read(
    State(pool): State<PgPool>,
    Query(query): Query<NotificationQuery>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = AuthService::verify_token(&query.token).await?;
    let notification_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound("Invalid notification ID".to_string()))?;
    let success = NotificationService::mark_read(&pool, claims.sub, notification_id).await;
    Ok(Json(serde_json::json!({ "success": success })))
}

pub async fn mark_all_read(
    State(pool): State<PgPool>,
    Query(query): Query<NotificationQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = AuthService::verify_token(&query.token).await?;
    let count = NotificationService::mark_all_read(&pool, claims.sub).await;
    Ok(Json(serde_json::json!({ "marked_count": count })))
}

pub async fn delete(
    State(pool): State<PgPool>,
    Query(query): Query<NotificationQuery>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = AuthService::verify_token(&query.token).await?;
    let notification_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound("Invalid notification ID".to_string()))?;
    let success = NotificationService::delete(&pool, claims.sub, notification_id).await;
    Ok(Json(serde_json::json!({ "success": success })))
}

// Notification Preferences handlers
pub async fn get_notification_preferences(
    State(pool): State<PgPool>,
    Query(query): Query<NotificationQuery>,
) -> AppResult<Json<NotificationPreferencesResponse>> {
    let claims = AuthService::verify_token(&query.token).await?;
    let preferences = NotificationService::get_notification_preferences(&pool, claims.sub).await;
    Ok(Json(NotificationPreferencesResponse { preferences }))
}

pub async fn update_notification_preferences(
    State(pool): State<PgPool>,
    Query(query): Query<NotificationQuery>,
    Json(req): Json<UpdateNotificationPreferencesRequest>,
) -> AppResult<Json<NotificationPreferencesResponse>> {
    let claims = AuthService::verify_token(&query.token).await?;
    let preferences = NotificationService::update_notification_preferences(&pool, claims.sub, req.preferences).await;
    Ok(Json(NotificationPreferencesResponse { preferences }))
}
