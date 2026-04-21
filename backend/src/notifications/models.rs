use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub message: Option<String>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub notifications: Vec<Notification>,
    pub unread_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferenceResponse {
    pub channel: String,
    pub enabled: bool,
    pub reminder_days_before: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferencesResponse {
    pub preferences: Vec<NotificationPreferenceResponse>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotificationPreferencesRequest {
    pub preferences: Vec<UpdatePreferenceRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferenceRequest {
    pub channel: String,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_response_with_empty_notifications() {
        let resp = NotificationResponse {
            notifications: vec![],
            unread_count: 0,
        };
        assert!(resp.notifications.is_empty());
        assert_eq!(resp.unread_count, 0);
    }

    #[test]
    fn notification_preference_response_fields() {
        let pref = NotificationPreferenceResponse {
            channel: "email".to_string(),
            enabled: true,
            reminder_days_before: 3,
        };
        assert_eq!(pref.channel, "email");
        assert!(pref.enabled);
        assert_eq!(pref.reminder_days_before, 3);
    }

    #[test]
    fn update_preference_request_deserialize() {
        let json = r#"{"channel":"push","enabled":true}"#;
        let req: UpdatePreferenceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.channel, "push");
        assert!(req.enabled);
    }

    #[test]
    fn update_notification_preferences_wrap() {
        let req = UpdateNotificationPreferencesRequest {
            preferences: vec![
                UpdatePreferenceRequest {
                    channel: "email".to_string(),
                    enabled: true,
                },
                UpdatePreferenceRequest {
                    channel: "push".to_string(),
                    enabled: false,
                },
            ],
        };
        assert_eq!(req.preferences.len(), 2);
    }

    #[test]
    fn notification_preferences_response_serialization() {
        let resp = NotificationPreferencesResponse {
            preferences: vec![NotificationPreferenceResponse {
                channel: "email".to_string(),
                enabled: false,
                reminder_days_before: 3,
            }],
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("preferences").is_some());
        let prefs = json.get("preferences").unwrap().as_array().unwrap();
        assert_eq!(prefs.len(), 1);
    }

    #[test]
    fn uuid_parse_valid() {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = Uuid::parse_str(id_str);
        assert!(result.is_ok());
    }

    #[test]
    fn uuid_parse_invalid() {
        let result = Uuid::parse_str("not-a-uuid");
        assert!(result.is_err());
    }
}
