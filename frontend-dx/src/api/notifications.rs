use crate::api::client::{api_get, api_post, api_delete, ApiError};
use crate::models::notification::*;

pub async fn list() -> Result<NotificationListResponse, ApiError> {
    api_get("/api/notifications").await
}

pub async fn mark_as_read(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_post(&format!("/api/notifications/{}/read", id), &serde_json::json!({})).await
}

pub async fn mark_all_as_read() -> Result<(), ApiError> {
    api_post("/api/notifications/read-all", &serde_json::json!({})).await
}

pub async fn delete(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/notifications/{}", id)).await
}