use axum::{
    extract::{Path, State},
    http::HeaderMap,
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
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::{AppError, AppResult};
use crate::shared::OptionalQuery;

#[derive(serde::Deserialize)]
pub struct NotificationQuery {
    pub token: Option<String>,
}

pub async fn list(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    headers: HeaderMap,
) -> AppResult<Json<NotificationResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;
    let notifications = NotificationService::list(&pool, claims.sub).await?;
    Ok(Json(notifications))
}

pub async fn mark_read(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;
    let notification_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound("Invalid notification ID".to_string()))?;
    let success = NotificationService::mark_read(&pool, claims.sub, notification_id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

pub async fn mark_all_read(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;
    let count = NotificationService::mark_all_read(&pool, claims.sub).await?;
    Ok(Json(serde_json::json!({ "marked_count": count })))
}

pub async fn delete(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;
    let notification_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound("Invalid notification ID".to_string()))?;
    let success = NotificationService::delete(&pool, claims.sub, notification_id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

// Notification Preferences handlers
pub async fn get_notification_preferences(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    headers: HeaderMap,
) -> AppResult<Json<NotificationPreferencesResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;
    let preferences = NotificationService::get_notification_preferences(&pool, claims.sub).await?;
    Ok(Json(NotificationPreferencesResponse { preferences }))
}

pub async fn update_notification_preferences(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    headers: HeaderMap,
    Json(req): Json<UpdateNotificationPreferencesRequest>,
) -> AppResult<Json<NotificationPreferencesResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;
    let preferences = NotificationService::update_notification_preferences(&pool, claims.sub, req.preferences).await?;
    Ok(Json(NotificationPreferencesResponse { preferences }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn notification_query_deserialize() {
        let query = "token=test-token";
        let result: NotificationQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("test-token"));
    }

    #[test]
    fn notification_query_empty() {
        let query = "";
        let result: NotificationQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_none());
    }

    #[test]
    fn parse_valid_uuid_from_path() {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = Uuid::parse_str(id_str);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_invalid_uuid_from_path() {
        let id_str = "not-a-valid-uuid";
        let result = Uuid::parse_str(id_str);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_uuid_creates_not_found_error() {
        // Simulates what the handler does: parse_str fails => NotFound error
        let id_str = "invalid";
        let result = Uuid::parse_str(id_str)
            .map_err(|_| AppError::NotFound("Invalid notification ID".to_string()));
        assert!(result.is_err());
        match result {
            Err(AppError::NotFound(msg)) => assert!(msg.contains("Invalid notification ID")),
            _ => panic!("Expected NotFound error"),
        }
    }
}