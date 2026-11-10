use crate::api::client::{api_get, api_post, api_put, ApiError};
use crate::models::notification::*;
use serde_json::Value;

pub async fn list() -> Result<Vec<NotificationChannel>, ApiError> {
    api_get("/api/settings/notification-channels").await
}

pub async fn update(
    provider: &str,
    req: &UpdateNotificationChannelRequest,
) -> Result<NotificationChannel, ApiError> {
    api_put(
        &format!("/api/settings/notification-channels/{provider}"),
        req,
    )
    .await
}

pub async fn test(provider: &str, config: Value) -> Result<(), ApiError> {
    api_post::<_, serde_json::Value>(
        &format!("/api/settings/notification-channels/{provider}/test"),
        &serde_json::json!({ "config": config }),
    )
    .await
    .map(|_| ())
}

pub async fn get_summary() -> Result<SummaryNotificationSettings, ApiError> {
    api_get("/api/settings/summary-notification").await
}

pub async fn update_summary(
    req: &UpdateSummaryNotificationSettingsRequest,
) -> Result<SummaryNotificationSettings, ApiError> {
    api_put("/api/settings/summary-notification", req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_paths() {
        assert_eq!(
            "/api/settings/notification-channels/telegram",
            "/api/settings/notification-channels/telegram"
        );
        assert_eq!(
            "/api/settings/notification-channels/ntfy/test",
            "/api/settings/notification-channels/ntfy/test"
        );
        assert_eq!(
            "/api/settings/summary-notification",
            "/api/settings/summary-notification"
        );
    }

    #[test]
    fn update_request_serializes() {
        let req = UpdateNotificationChannelRequest {
            enabled: true,
            config: serde_json::json!({"chat_id": "1"}),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("chat_id"));
    }
}
