use sqlx::PgPool;
use uuid::Uuid;

use crate::notifications::models::{
    Notification, NotificationPreferenceResponse, NotificationResponse,
    UpdatePreferenceRequest,
};

pub struct NotificationService;

impl NotificationService {
    pub async fn list(pool: &PgPool, user_id: Uuid) -> NotificationResponse {
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
        .await
        .unwrap_or_default();

        let unread_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        NotificationResponse {
            notifications,
            unread_count,
        }
    }

    pub async fn mark_read(pool: &PgPool, user_id: Uuid, id: Uuid) -> bool {
        let result = sqlx::query(
            "UPDATE notifications SET is_read = true WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await;

        result.map(|r| r.rows_affected() > 0).unwrap_or(false)
    }

    pub async fn mark_all_read(pool: &PgPool, user_id: Uuid) -> i64 {
        let result = sqlx::query(
            "UPDATE notifications SET is_read = true WHERE user_id = $1 AND is_read = false",
        )
        .bind(user_id)
        .execute(pool)
        .await;

        result.map(|r| r.rows_affected() as i64).unwrap_or(0)
    }

    pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> bool {
        let result = sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await;

        result.map(|r| r.rows_affected() > 0).unwrap_or(false)
    }

    pub async fn get_notification_preferences(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Vec<NotificationPreferenceResponse> {
        let prefs = sqlx::query_as::<_, (String, bool, i32)>(
            "SELECT channel, enabled, reminder_days_before FROM notification_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        if prefs.is_empty() {
            return vec![
                NotificationPreferenceResponse { channel: "email".to_string(), enabled: true, reminder_days_before: 3 },
                NotificationPreferenceResponse { channel: "push".to_string(), enabled: true, reminder_days_before: 3 },
                NotificationPreferenceResponse { channel: "system".to_string(), enabled: true, reminder_days_before: 3 },
            ];
        }

        prefs
            .into_iter()
            .map(|(channel, enabled, reminder_days_before)| NotificationPreferenceResponse {
                channel,
                enabled,
                reminder_days_before,
            })
            .collect()
    }

    pub async fn update_notification_preferences(
        pool: &PgPool,
        user_id: Uuid,
        updates: Vec<UpdatePreferenceRequest>,
    ) -> Vec<NotificationPreferenceResponse> {
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
            .await
            .ok();
        }

        Self::get_notification_preferences(pool, user_id).await
    }
}
