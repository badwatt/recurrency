use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationListResponse {
    pub notifications: Vec<Notification>,
    pub unread_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationPreference {
    pub channel: String,
    pub enabled: bool,
    pub reminder_days_before: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationPreferencesResponse {
    pub preferences: Vec<NotificationPreference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateNotificationPreferenceRequest {
    pub channel: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateNotificationPreferencesRequest {
    pub preferences: Vec<UpdateNotificationPreferenceRequest>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_serialization() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "upcoming_payment".into(),
            title: "Netflix due tomorrow".into(),
            message: Some("Your Netflix subscription of €15.99 is due tomorrow.".into()),
            is_read: false,
            created_at: DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&n).unwrap();
        assert!(json.contains("\"type\""));
        let back: Notification = serde_json::from_str(&json).unwrap();
        assert_eq!(back.notification_type, "upcoming_payment");
    }

    #[test]
    fn list_response_unread_count() {
        let resp = NotificationListResponse {
            notifications: vec![],
            unread_count: 3,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: NotificationListResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.unread_count, 3);
    }

    #[test]
    fn notification_type_rename() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "overdue_payment".into(),
            title: "Overdue!".into(),
            message: None,
            is_read: true,
            created_at: DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&n).unwrap();
        // The field should be serialized as "type" due to serde rename
        assert!(json.contains("\"type\""));
        assert!(!json.contains("notification_type"));
    }

    #[test]
    fn notification_preference_roundtrip() {
        let pref = NotificationPreference {
            channel: "email".into(),
            enabled: true,
            reminder_days_before: 3,
        };
        let json = serde_json::to_string(&pref).unwrap();
        let back: NotificationPreference = serde_json::from_str(&json).unwrap();
        assert_eq!(back.channel, "email");
        assert!(back.enabled);
        assert_eq!(back.reminder_days_before, 3);
    }

    #[test]
    fn notification_preferences_response() {
        let resp = NotificationPreferencesResponse {
            preferences: vec![
                NotificationPreference {
                    channel: "email".into(),
                    enabled: true,
                    reminder_days_before: 3,
                },
                NotificationPreference {
                    channel: "push".into(),
                    enabled: false,
                    reminder_days_before: 1,
                },
            ],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: NotificationPreferencesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.preferences.len(), 2);
    }

    #[test]
    fn update_notification_preferences_request() {
        let req = UpdateNotificationPreferencesRequest {
            preferences: vec![
                UpdateNotificationPreferenceRequest {
                    channel: "email".into(),
                    enabled: true,
                },
            ],
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdateNotificationPreferencesRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.preferences.len(), 1);
        assert_eq!(back.preferences[0].channel, "email");
    }

    #[test]
    fn notification_with_message() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "upcoming_payment".into(),
            title: "Due soon".into(),
            message: Some("Your subscription is due".into()),
            is_read: false,
            created_at: DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&n).unwrap();
        let back: Notification = serde_json::from_str(&json).unwrap();
        assert_eq!(back.message.as_deref(), Some("Your subscription is due"));
    }
}