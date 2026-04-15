use crate::api::client::{api_get, api_put, ApiError};
use crate::models::notification::*;

pub async fn get() -> Result<NotificationPreferencesResponse, ApiError> {
    api_get("/api/settings/notifications").await
}

pub async fn update(req: &UpdateNotificationPreferencesRequest) -> Result<NotificationPreferencesResponse, ApiError> {
    api_put("/api/settings/notifications", req).await
}