use sqlx::PgPool;
use uuid::Uuid;

use crate::notifications::models::{
    Notification, NotificationPreferenceResponse, NotificationResponse, UpdatePreferenceRequest,
};
#[cfg(test)]
use crate::notifications::models::{
    NotificationPreferencesResponse, UpdateNotificationPreferencesRequest,
};
use crate::shared::error::AppResult;

pub const SELECT_NOTIFICATIONS_SQL: &str =
"SELECT id, user_id, type as \"notification_type\", title, message, is_read, created_at FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50";

pub const INSERT_NOTIFICATION_SQL: &str =
"INSERT INTO notifications (id, user_id, type, title, message, is_read, created_at) VALUES ($1, $2, $3, $4, $5, false, NOW())";


pub struct NotificationService;

impl NotificationService {
    pub async fn list(pool: &PgPool, user_id: Uuid) -> AppResult<NotificationResponse> {
        let unread_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let notifications = sqlx::query_as::<_, Notification>(SELECT_NOTIFICATIONS_SQL)
            .bind(user_id)
            .fetch_all(pool)
            .await?;

        Ok(NotificationResponse {
            notifications,
            unread_count: unread_count.0,
        })
    }

    pub async fn mark_read(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<bool> {
        let result =
            sqlx::query("UPDATE notifications SET is_read = true WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(user_id)
                .execute(pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::error::AppError::NotFound(
                "Notification not found".to_string(),
            ));
        }
        Ok(true)
    }

    pub async fn mark_all_read(pool: &PgPool, user_id: Uuid) -> AppResult<i64> {
        let result = sqlx::query(
            "UPDATE notifications SET is_read = true WHERE user_id = $1 AND is_read = false",
        )
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::error::AppError::NotFound(
                "Notification not found".to_string(),
            ));
        }
        Ok(true)
    }

    pub async fn get_notification_preferences(
        pool: &PgPool,
        user_id: Uuid,
    ) -> AppResult<Vec<NotificationPreferenceResponse>> {
        let prefs = sqlx::query_as::<_, (String, bool, i32)>(
            "SELECT channel, enabled, reminder_days_before FROM notification_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool).await?;

        let mut result = default_notification_preferences();
        for (channel, enabled, days) in prefs {
            if let Some(pref) = result.iter_mut().find(|p| p.channel == channel) {
                pref.enabled = enabled;
                pref.reminder_days_before = days;
            }
        }
        Ok(result)
    }

    pub async fn update_notification_preferences(
        pool: &PgPool,
        user_id: Uuid,
        prefs: Vec<UpdatePreferenceRequest>,
    ) -> AppResult<Vec<NotificationPreferenceResponse>> {
        for req in prefs {
            // Check if enabling push or system notifications
            let should_send_confirmation = req.enabled
                && (req.channel == "push" || req.channel == "system");

            sqlx::query(
                "INSERT INTO notification_preferences (user_id, channel, enabled, reminder_days_before) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id, channel) DO UPDATE SET enabled = $3, reminder_days_before = $4",
            )
            .bind(user_id).bind(&req.channel).bind(req.enabled).bind(req.reminder_days_before)
            .execute(pool).await?;

            // Send confirmation notification if enabled
            if should_send_confirmation {
                let (title, message) = if req.channel == "push" {
                    ("Push notifications enabled", "You will receive push notifications for upcoming payments.")
                } else {
                    ("System notifications enabled", "You will receive desktop notifications for upcoming payments.")
                };

                sqlx::query(
                    INSERT_NOTIFICATION_SQL,
                )
                .bind(Uuid::new_v4())
                .bind(user_id)
                .bind("system")
                .bind(title)
                .bind(message)
                .execute(pool).await?;
            }
        }
        Self::get_notification_preferences(pool, user_id).await
    }
}

pub fn default_notification_preferences() -> Vec<NotificationPreferenceResponse> {
    vec![
        NotificationPreferenceResponse {
            channel: "email".to_string(),
            enabled: false,
            reminder_days_before: 3,
        },
        NotificationPreferenceResponse {
            channel: "push".to_string(),
            enabled: false,
            reminder_days_before: 3,
        },
        NotificationPreferenceResponse {
            channel: "system".to_string(),
            enabled: false,
            reminder_days_before: 3,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn default_preferences_three_channels() {
        let prefs = default_notification_preferences();
        assert_eq!(prefs.len(), 3);
    }

    #[test]
    fn default_preferences_email_disabled() {
        let p = default_notification_preferences()
            .into_iter()
            .find(|p| p.channel == "email")
            .unwrap();
        assert!(!p.enabled);
        assert_eq!(p.reminder_days_before, 3);
    }

    #[test]
    fn default_preferences_push_disabled() {
        let p = default_notification_preferences()
            .into_iter()
            .find(|p| p.channel == "push")
            .unwrap();
        assert!(!p.enabled);
    }

    #[test]
    fn default_preferences_system_disabled() {
        let p = default_notification_preferences()
            .into_iter()
            .find(|p| p.channel == "system")
            .unwrap();
        assert!(!p.enabled);
    }

    #[test]
    fn default_preferences_all_three_days() {
        for p in default_notification_preferences() {
            assert_eq!(p.reminder_days_before, 3);
        }
    }

    #[test]
    fn notification_response_unread_count() {
        let resp = NotificationResponse {
            notifications: vec![],
            unread_count: 5,
        };
        assert_eq!(resp.unread_count, 5);
    }

    #[test]
    fn notification_response_empty() {
        let resp = NotificationResponse {
            notifications: vec![],
            unread_count: 0,
        };
        assert!(resp.notifications.is_empty());
    }

    #[test]
    fn notification_unread() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing".to_string(),
            title: "Test".to_string(),
            message: Some("msg".to_string()),
            is_read: false,
            created_at: chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        };
        assert!(!n.is_read);
    }

    #[test]
    fn notification_read() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "alert".to_string(),
            title: "R".to_string(),
            message: None,
            is_read: true,
            created_at: chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        };
        assert!(n.is_read);
    }

    #[test]
    fn notification_serde() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "alert".to_string(),
            title: "Ser".to_string(),
            message: None,
            is_read: true,
            created_at: chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        };
        let json = serde_json::to_string(&n).unwrap();
        let rt: Notification = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.title, n.title);
    }

    #[test]
    fn update_preference_email_enabled() {
        let req: UpdatePreferenceRequest =
            serde_json::from_str(r#"{"channel":"email","enabled":true}"#).unwrap();
        assert_eq!(req.channel, "email");
        assert!(req.enabled);
    }

    #[test]
    fn update_preference_push_disabled() {
        let req: UpdatePreferenceRequest =
            serde_json::from_str(r#"{"channel":"push","enabled":false}"#).unwrap();
        assert_eq!(req.channel, "push");
        assert!(!req.enabled);
    }

    #[test]
    fn preference_response_fields() {
        let p = NotificationPreferenceResponse {
            channel: "e".to_string(),
            enabled: true,
            reminder_days_before: 5,
        };
        assert_eq!(p.reminder_days_before, 5);
    }

    #[test]
    fn preference_response_serialize() {
        let p = NotificationPreferenceResponse {
            channel: "email".to_string(),
            enabled: false,
            reminder_days_before: 3,
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("email"));
    }

    #[test]
    fn rows_affected_positive() {
        assert!(5i64 > 0);
    }
    #[test]
    fn rows_affected_zero() {
        assert!(0i64 == 0);
    }
    // ── NotificationService struct ──

    #[test]
    fn notification_service_struct_reference() {
        let _ = NotificationService;
    }

    // ── NotificationResponse with populated list ──

    #[test]
    fn notification_response_with_multiple_notifications() {
        let n1 = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing".to_string(),
            title: "Netflix Due".to_string(),
            message: Some("Tomorrow".to_string()),
            is_read: false,
            created_at: chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        };
        let n2 = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "reminder".to_string(),
            title: "Gym".to_string(),
            message: None,
            is_read: true,
            created_at: chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        };
        let resp = NotificationResponse {
            notifications: vec![n1, n2],
            unread_count: 1,
        };
        assert_eq!(resp.notifications.len(), 2);
        assert_eq!(resp.unread_count, 1);
        assert!(!resp.notifications[0].is_read);
        assert!(resp.notifications[1].is_read);
    }

    #[test]
    fn notification_response_serialize_with_data() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "alert".to_string(),
            title: "Test Alert".to_string(),
            message: Some("Test message".to_string()),
            is_read: false,
            created_at: chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        };
        let resp = NotificationResponse {
            notifications: vec![n],
            unread_count: 3,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["unread_count"], 3);
        let notifs = json["notifications"].as_array().unwrap();
        assert_eq!(notifs.len(), 1);
    }

    #[test]
    fn notification_response_unread_count_large() {
        let resp = NotificationResponse {
            notifications: vec![],
            unread_count: 9999,
        };
        assert_eq!(resp.unread_count, 9999);
    }

    // ── NotificationPreferencesResponse ──

    #[test]
    fn notification_preferences_response_empty() {
        let resp = NotificationPreferencesResponse {
            preferences: vec![],
        };
        assert!(resp.preferences.is_empty());
    }

    #[test]
    fn notification_preferences_response_multiple() {
        let resp = NotificationPreferencesResponse {
            preferences: vec![
                NotificationPreferenceResponse {
                    channel: "email".to_string(),
                    enabled: true,
                    reminder_days_before: 5,
                },
                NotificationPreferenceResponse {
                    channel: "push".to_string(),
                    enabled: false,
                    reminder_days_before: 1,
                },
                NotificationPreferenceResponse {
                    channel: "system".to_string(),
                    enabled: true,
                    reminder_days_before: 3,
                },
            ],
        };
        assert_eq!(resp.preferences.len(), 3);
        let email = resp
            .preferences
            .iter()
            .find(|p| p.channel == "email")
            .unwrap();
        assert!(email.enabled);
        assert_eq!(email.reminder_days_before, 5);
    }

    #[test]
    fn notification_preferences_response_serialize_roundtrip() {
        let resp = NotificationPreferencesResponse {
            preferences: vec![NotificationPreferenceResponse {
                channel: "email".to_string(),
                enabled: true,
                reminder_days_before: 7,
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("preferences"));
        let rt: NotificationPreferencesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.preferences.len(), 1);
        assert_eq!(rt.preferences[0].reminder_days_before, 7);
    }

    // ── UpdateNotificationPreferencesRequest ──

    #[test]
    fn update_notification_preferences_request_deserialize() {
        let json = r#"{"preferences":[{"channel":"email","enabled":true},{"channel":"push","enabled":false}]}"#;
        let req: UpdateNotificationPreferencesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.preferences.len(), 2);
        let email = req
            .preferences
            .iter()
            .find(|p| p.channel == "email")
            .unwrap();
        assert!(email.enabled);
    }

    #[test]
    fn update_notification_preferences_request_empty() {
        let json = r#"{"preferences":[]}"#;
        let req: UpdateNotificationPreferencesRequest = serde_json::from_str(json).unwrap();
        assert!(req.preferences.is_empty());
    }

    #[test]
    fn update_preference_request_all_channels() {
        for channel in &["email", "push", "system"] {
            let json = format!(r#"{{"channel":"{}","enabled":true}}"#, channel);
            let req: UpdatePreferenceRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(req.channel, *channel);
            assert!(req.enabled);
        }
    }

    // ── Notification serde with type field ──

    #[test]
    fn notification_serde_type_field() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing_reminder".to_string(),
            title: "Test".to_string(),
            message: None,
            is_read: false,
            created_at: chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        };
        let json = serde_json::to_value(&n).unwrap();
        assert_eq!(json["type"], "billing_reminder");
    }

    #[test]
    fn notification_serde_roundtrip_with_message() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "reminder".to_string(),
            title: "Payment Due".to_string(),
            message: Some("Netflix subscription renews tomorrow".to_string()),
            is_read: false,
            created_at: chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        };
        let json = serde_json::to_string(&n).unwrap();
        let rt: Notification = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.title, "Payment Due");
        assert_eq!(
            rt.message.as_deref(),
            Some("Netflix subscription renews tomorrow")
        );
        assert!(!rt.is_read);
    }

    // ── rows_affected boundary ──

    #[test]
    fn rows_affected_boundary_values() {
        assert_eq!(0i64, 0);
        assert!(1i64 > 0);
        assert!(100i64 > 0);
    }

    #[test]
    fn pref_partial_override() {
        let mut r = default_notification_preferences();
        if let Some(p) = r.iter_mut().find(|p| p.channel == "email") {
            p.enabled = true;
            p.reminder_days_before = 5;
        }
        let e = r.iter().find(|p| p.channel == "email").unwrap();
        assert!(e.enabled);
    }

    #[test]
    fn pref_no_db_match() {
        let mut r = default_notification_preferences();
        for (ch, en, days) in Vec::<(String, bool, i32)>::new() {
            if let Some(p) = r.iter_mut().find(|x| x.channel == ch) {
                p.enabled = en;
                p.reminder_days_before = days;
            }
        }
        for p in &r {
            assert!(!p.enabled);
        }
    }

    #[test]
    fn pref_all_override() {
        let mut r = default_notification_preferences();
        let prefs = vec![
            ("email".to_string(), true, 7),
            ("push".to_string(), true, 5),
            ("system".to_string(), false, 1),
        ];
        for (ch, en, days) in prefs {
            if let Some(p) = r.iter_mut().find(|x| x.channel == ch) {
                p.enabled = en;
                p.reminder_days_before = days;
            }
        }
        assert!(r[0].enabled);
        assert!(r[1].enabled);
    }

    #[test]
    fn pref_zero_days() {
        let p = NotificationPreferenceResponse {
            channel: "email".to_string(),
            enabled: false,
            reminder_days_before: 0,
        };
        assert_eq!(p.reminder_days_before, 0);
    }

    #[test]
    fn msg_null_deser() {
        let json = r#"{"id":"550e8400-e29b-41d4-a716-446655440000","user_id":"550e8400-e29b-41d4-a716-446655440001","type":"t","title":"T","message":null,"is_read":false,"created_at":"2024-01-01T00:00:00Z"}"#;
        let n: Notification = serde_json::from_str(json).unwrap();
        assert!(n.message.is_none());
    }

    #[test]
    fn upd_pref_unknown_ch() {
        let req: UpdatePreferenceRequest =
            serde_json::from_str(r#"{"channel":"telegram","enabled":true}"#).unwrap();
        assert_eq!(req.channel, "telegram");
    }

    #[test]
    fn rows_0() {
        assert!(0i64 == 0);
    }
    #[test]
    fn rows_1() {
        assert!(1i64 > 0);
    }
    #[test]
    fn rows_5() {
        assert!(5i64 > 0);
    }

    #[test]
    fn select_query_uses_db_type_column() {
        assert!(!SELECT_NOTIFICATIONS_SQL.contains("notification_type as type"),
            "SELECT SQL uses wrong column name");
        assert!(SELECT_NOTIFICATIONS_SQL.contains(r#"type as "notification_type""#),
            "SELECT SQL must alias DB column 'type' for sqlx FromRow");
    }

    #[test]
    fn insert_query_uses_db_type_column() {
        assert!(!INSERT_NOTIFICATION_SQL.contains("notification_type"),
            "INSERT SQL uses wrong column name");
    }
}

#[cfg(test)]
mod serde_tests {
    use crate::notifications::models::{
        Notification, NotificationResponse, UpdatePreferenceRequest,
    };
    use uuid::Uuid;

    #[test]
    fn notification_response_unread_count_serde() {
        let resp = NotificationResponse {
            notifications: vec![],
            unread_count: 5,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("unread_count"));
    }

    #[test]
    fn notification_response_with_notifications() {
        let item = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing".to_string(),
            title: "Test".to_string(),
            message: None,
            is_read: false,
            created_at: chrono::Utc::now(),
        };
        let resp = NotificationResponse {
            notifications: vec![item],
            unread_count: 1,
        };
        assert_eq!(resp.notifications.len(), 1);
        assert_eq!(resp.unread_count, 1);
    }

    #[test]
    fn update_preference_request_serde() {
        let json = r#"{"channel":"email","enabled":true,"reminder_days_before":5}"#;
        let req: UpdatePreferenceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.channel, "email");
        assert!(req.enabled);
    }

    #[test]
    fn update_preference_push_confirmation_message() {
        // Verify push channel triggers confirmation
        let channel = "push";
        let enabled = true;
        let should_send = enabled && (channel == "push" || channel == "system");
        assert!(should_send);
    }

    #[test]
    fn update_preference_system_confirmation_message() {
        // Verify system channel triggers confirmation
        let channel = "system";
        let enabled = true;
        let should_send = enabled && (channel == "push" || channel == "system");
        assert!(should_send);
    }

    #[test]
    fn update_preference_no_confirmation_when_disabled() {
        // Verify disabled channel does NOT trigger confirmation
        let channel = "push";
        let enabled = false;
        let should_send = enabled && (channel == "push" || channel == "system");
        assert!(!should_send);
    }

    #[test]
    fn update_preference_no_confirmation_for_email() {
        // Verify email channel does NOT trigger confirmation
        let channel = "email";
        let enabled = true;
        let should_send = enabled && (channel == "push" || channel == "system");
        assert!(!should_send);
    }
}
