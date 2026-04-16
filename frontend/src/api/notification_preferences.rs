use crate::api::client::{api_get, api_put, ApiError};
use crate::models::notification::*;

pub async fn get() -> Result<NotificationPreferencesResponse, ApiError> {
    api_get("/api/settings/notifications").await
}

pub async fn update(req: &UpdateNotificationPreferencesRequest) -> Result<NotificationPreferencesResponse, ApiError> {
    api_put("/api/settings/notifications", req).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn notification_preferences_api_path() {
        let path = "/api/settings/notifications";
        assert!(path.starts_with("/api/"));
        assert!(path.contains("notifications"));
    }

    #[test]
    fn update_preferences_request_serialization() {
        use crate::models::notification::{
            UpdateNotificationPreferenceRequest,
            UpdateNotificationPreferencesRequest,
        };
        let req = UpdateNotificationPreferencesRequest {
            preferences: vec![
                UpdateNotificationPreferenceRequest {
                    channel: "email".into(),
                    enabled: true,
                },
            ],
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("email"));
        assert!(json.contains("true"));
    }
}