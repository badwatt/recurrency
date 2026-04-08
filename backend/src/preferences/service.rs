use sqlx::PgPool;
use uuid::Uuid;

use crate::preferences::models::{PreferencesResponse, UpdatePreferencesRequest};

pub struct PreferencesService;

impl PreferencesService {
    pub async fn get_preferences(pool: &PgPool, user_id: Uuid) -> PreferencesResponse {
        let prefs = sqlx::query_as::<_, (String, String)>(
            "SELECT key, value FROM user_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut response = PreferencesResponse::default();

        for (key, value) in prefs {
            match key.as_str() {
                "theme" => response.theme = value,
                "subscriptions_view" => response.subscriptions_view = value,
                _ => {}
            }
        }

        response
    }

    pub async fn update_preferences(
        pool: &PgPool,
        user_id: Uuid,
        req: UpdatePreferencesRequest,
    ) -> PreferencesResponse {
        if let Some(theme) = req.theme {
            sqlx::query(
                r#"
                INSERT INTO user_preferences (user_id, key, value)
                VALUES ($1, 'theme', $2)
                ON CONFLICT (user_id, key) DO UPDATE SET value = $2
                "#,
            )
            .bind(user_id)
            .bind(&theme)
            .execute(pool)
            .await
            .ok();
        }

        if let Some(subscriptions_view) = req.subscriptions_view {
            sqlx::query(
                r#"
                INSERT INTO user_preferences (user_id, key, value)
                VALUES ($1, 'subscriptions_view', $2)
                ON CONFLICT (user_id, key) DO UPDATE SET value = $2
                "#,
            )
            .bind(user_id)
            .bind(&subscriptions_view)
            .execute(pool)
            .await
            .ok();
        }

        Self::get_preferences(pool, user_id).await
    }
}
