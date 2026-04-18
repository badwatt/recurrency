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
    use crate::models::notification::{
        NotificationPreferencesResponse,
        NotificationPreference,
        UpdateNotificationPreferencesRequest,
        UpdateNotificationPreferenceRequest,
        NotificationListResponse,
        Notification,
    };
    use uuid::Uuid;
    use chrono::DateTime;

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

    #[test]
    fn notification_preferences_response_fields() {
        let resp = NotificationPreferencesResponse {
            preferences: vec![
                NotificationPreference {
                    channel: "email".into(),
                    enabled: true,
                    reminder_days_before: 3,
                },
            ],
        };
        assert_eq!(resp.preferences.len(), 1);
        assert_eq!(resp.preferences[0].channel, "email");
    }

    #[test]
    fn notification_preferences_response_multiple() {
        let resp = NotificationPreferencesResponse {
            preferences: vec![
                NotificationPreference { channel: "email".into(), enabled: true, reminder_days_before: 3 },
                NotificationPreference { channel: "push".into(), enabled: false, reminder_days_before: 1 },
                NotificationPreference { channel: "browser".into(), enabled: true, reminder_days_before: 7 },
            ],
        };
        assert_eq!(resp.preferences.len(), 3);
    }

    #[test]
    fn update_request_push_disabled() {
        let req = UpdateNotificationPreferencesRequest {
            preferences: vec![
                UpdateNotificationPreferenceRequest {
                    channel: "push".into(),
                    enabled: false,
                },
            ],
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("push"));
        assert!(json.contains("false"));
    }

    #[test]
    fn update_request_all_channels() {
        let channels = vec!["email", "push", "browser"];
        for ch in channels {
            let req = UpdateNotificationPreferencesRequest {
                preferences: vec![
                    UpdateNotificationPreferenceRequest {
                        channel: ch.into(),
                        enabled: true,
                    },
                ],
            };
            let json = serde_json::to_string(&req).unwrap();
            assert!(json.contains(ch));
        }
    }

    #[test]
    fn notification_list_response_empty() {
        let resp = NotificationListResponse { notifications: vec![], unread_count: 0 };
        assert!(resp.notifications.is_empty());
        assert_eq!(resp.unread_count, 0);
    }

    #[test]
    fn notification_list_response_with_notifications() {
        let resp = NotificationListResponse {
            notifications: vec![
                Notification {
                    id: Uuid::new_v4(),
                    user_id: Uuid::new_v4(),
                    notification_type: "reminder".into(),
                    title: "Test".into(),
                    message: None,
                    is_read: false,
                    created_at: DateTime::UNIX_EPOCH,
                },
            ],
            unread_count: 1,
        };
        assert_eq!(resp.notifications.len(), 1);
        assert_eq!(resp.unread_count, 1);
    }
}