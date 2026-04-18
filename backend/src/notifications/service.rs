use sqlx::PgPool;
use uuid::Uuid;

use crate::notifications::models::{
    Notification, NotificationPreferenceResponse, NotificationResponse,
    UpdatePreferenceRequest,
};
use crate::shared::error::AppResult;

pub struct NotificationService;

impl NotificationService {
    pub async fn list(pool: &PgPool, user_id: Uuid) -> AppResult<NotificationResponse> {
        let notifications = sqlx::query_as::<_, Notification>(
            r#"
            SELECT id, user_id, type as "type", title, message, is_read, created_at
            FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT 50
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let unread_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(NotificationResponse {
            notifications,
            unread_count,
        })
    }

    pub async fn mark_read(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE notifications SET is_read = true WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
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

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_notification_preferences(
        pool: &PgPool,
        user_id: Uuid,
    ) -> AppResult<Vec<NotificationPreferenceResponse>> {
        let prefs = sqlx::query_as::<_, (String, bool, i32)>(
            "SELECT channel, enabled, reminder_days_before FROM notification_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        if prefs.is_empty() {
            return Ok(default_notification_preferences());
        }

        Ok(prefs
            .into_iter()
            .map(|(channel, enabled, reminder_days_before)| NotificationPreferenceResponse {
                channel,
                enabled,
                reminder_days_before,
            })
            .collect())
    }

    pub async fn update_notification_preferences(
        pool: &PgPool,
        user_id: Uuid,
        updates: Vec<UpdatePreferenceRequest>,
    ) -> AppResult<Vec<NotificationPreferenceResponse>> {
        for update in &updates {
            sqlx::query(
                r#"
                INSERT INTO notification_preferences (id, user_id, channel, enabled, reminder_days_before)
                VALUES ($1, $2, $3, $4, 3)
                ON CONFLICT (user_id, channel)
                DO UPDATE SET enabled = $4, updated_at = NOW()
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind(&update.channel)
            .bind(update.enabled)
            .execute(pool)
            .await?;
        }

        Self::get_notification_preferences(pool, user_id).await
    }
}

/// Generate default notification preferences when none exist in DB.
pub(crate) fn default_notification_preferences() -> Vec<NotificationPreferenceResponse> {
    vec![
        NotificationPreferenceResponse { channel: "email".to_string(), enabled: false, reminder_days_before: 3 },
        NotificationPreferenceResponse { channel: "push".to_string(), enabled: false, reminder_days_before: 3 },
        NotificationPreferenceResponse { channel: "system".to_string(), enabled: false, reminder_days_before: 3 },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn default_notification_preferences_has_three_channels() {
        let prefs = default_notification_preferences();
        assert_eq!(prefs.len(), 3);
    }

    #[test]
    fn default_notification_preferences_channels() {
        let prefs = default_notification_preferences();
        let channels: Vec<&str> = prefs.iter().map(|p| p.channel.as_str()).collect();
        assert!(channels.contains(&"email"));
        assert!(channels.contains(&"push"));
        assert!(channels.contains(&"system"));
    }

    #[test]
    fn default_notification_preferences_all_disabled() {
        let prefs = default_notification_preferences();
        for pref in &prefs {
            assert!(!pref.enabled, "{} should be disabled by default", pref.channel);
        }
    }

    #[test]
    fn default_notification_preferences_reminder_days() {
        let prefs = default_notification_preferences();
        for pref in &prefs {
            assert_eq!(pref.reminder_days_before, 3);
        }
    }

    // ── Notification filtering logic ──

    fn filter_by_type<'a>(notifs: &'a [Notification], notif_type: &str) -> Vec<&'a Notification> {
        notifs.iter().filter(|n| n.notification_type == notif_type).collect()
    }

    #[test]
    fn filter_by_type_reminders() {
        let notifs = vec![
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "reminder".to_string(), title: "R1".to_string(), message: None, is_read: false, created_at: Utc::now() },
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "alert".to_string(), title: "A1".to_string(), message: None, is_read: false, created_at: Utc::now() },
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "reminder".to_string(), title: "R2".to_string(), message: None, is_read: false, created_at: Utc::now() },
        ];
        let reminders = filter_by_type(&notifs, "reminder");
        assert_eq!(reminders.len(), 2);
    }

    #[test]
    fn filter_by_type_no_match() {
        let notifs = vec![
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "reminder".to_string(), title: "R1".to_string(), message: None, is_read: false, created_at: Utc::now() },
        ];
        let alerts = filter_by_type(&notifs, "alert");
        assert!(alerts.is_empty());
    }

    #[test]
    fn filter_by_type_empty() {
        let notifs: Vec<Notification> = vec![];
        let filtered = filter_by_type(&notifs, "reminder");
        assert!(filtered.is_empty());
    }

    // ── Unread count logic ──

    fn count_unread(notifs: &[Notification]) -> usize {
        notifs.iter().filter(|n| !n.is_read).count()
    }

    #[test]
    fn count_unread_all_unread() {
        let notifs = vec![
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "reminder".to_string(), title: "R1".to_string(), message: None, is_read: false, created_at: Utc::now() },
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "alert".to_string(), title: "A1".to_string(), message: None, is_read: false, created_at: Utc::now() },
        ];
        assert_eq!(count_unread(&notifs), 2);
    }

    #[test]
    fn count_unread_all_read() {
        let notifs = vec![
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "reminder".to_string(), title: "R1".to_string(), message: None, is_read: true, created_at: Utc::now() },
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "alert".to_string(), title: "A1".to_string(), message: None, is_read: true, created_at: Utc::now() },
        ];
        assert_eq!(count_unread(&notifs), 0);
    }

    #[test]
    fn count_unread_mixed() {
        let notifs = vec![
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "reminder".to_string(), title: "R1".to_string(), message: None, is_read: true, created_at: Utc::now() },
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "alert".to_string(), title: "A1".to_string(), message: None, is_read: false, created_at: Utc::now() },
        ];
        assert_eq!(count_unread(&notifs), 1);
    }

    #[test]
    fn count_unread_empty() {
        let notifs: Vec<Notification> = vec![];
        assert_eq!(count_unread(&notifs), 0);
    }

    // ── Mark read logic ──

    fn mark_all_read(notifs: &mut [Notification]) -> usize {
        let before = count_unread(notifs);
        for n in notifs.iter_mut() {
            n.is_read = true;
        }
        before
    }

    #[test]
    fn mark_all_read_returns_count() {
        let mut notifs = vec![
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "reminder".to_string(), title: "R1".to_string(), message: None, is_read: false, created_at: Utc::now() },
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "alert".to_string(), title: "A1".to_string(), message: None, is_read: false, created_at: Utc::now() },
        ];
        let count = mark_all_read(&mut notifs);
        assert_eq!(count, 2);
    }

    #[test]
    fn mark_all_read_updates() {
        let mut notifs = vec![
            Notification { id: Uuid::new_v4(), user_id: Uuid::new_v4(), notification_type: "reminder".to_string(), title: "R1".to_string(), message: None, is_read: false, created_at: Utc::now() },
        ];
        mark_all_read(&mut notifs);
        assert!(notifs[0].is_read);
    }

    // ── Reminder days calculation ──

    fn days_until_billing(billing_date: chrono::NaiveDate, today: chrono::NaiveDate) -> i64 {
        (billing_date - today).num_days()
    }

    #[test]
    fn days_until_billing_future() {
        let today = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let billing = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(days_until_billing(billing, today), 14);
    }

    #[test]
    fn days_until_billing_today() {
        let today = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let billing = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(days_until_billing(billing, today), 0);
    }

    #[test]
    fn days_until_billing_past() {
        let today = chrono::NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
        let billing = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(days_until_billing(billing, today), -5);
    }

    // ── Should remind logic ──

    fn should_remind(days_until: i64, reminder_days: i64) -> bool {
        days_until >= 0 && days_until <= reminder_days
    }

    #[test]
    fn should_remind_within_window() {
        assert!(should_remind(3, 3));
        assert!(should_remind(1, 3));
    }

    #[test]
    fn should_remind_today() {
        assert!(should_remind(0, 3));
    }

    #[test]
    fn should_not_remind_outside_window() {
        assert!(!should_remind(4, 3));
        assert!(!should_remind(10, 3));
    }

    #[test]
    fn should_not_remind_past() {
        assert!(!should_remind(-1, 3));
    }

    // ── Notification preference response construction ──

    #[test]
    fn notification_preference_response_structure() {
        let resp = NotificationPreferenceResponse {
            channel: "email".to_string(),
            enabled: true,
            reminder_days_before: 5,
        };
        assert_eq!(resp.channel, "email");
        assert!(resp.enabled);
        assert_eq!(resp.reminder_days_before, 5);
    }
}