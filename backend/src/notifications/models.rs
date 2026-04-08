use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub notifications: Vec<Notification>,
    pub unread_count: i64,
}

#[derive(Debug, Serialize)]
pub struct NotificationPreferenceResponse {
    pub channel: String,
    pub enabled: bool,
    pub reminder_days_before: i32,
}

#[derive(Debug, Serialize)]
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
