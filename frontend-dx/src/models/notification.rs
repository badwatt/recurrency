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
}