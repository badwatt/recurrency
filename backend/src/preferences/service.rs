use sqlx::PgPool;
use uuid::Uuid;

use crate::preferences::models::{PreferencesResponse, UpdatePreferencesRequest};
use crate::shared::error::AppResult;

pub struct PreferencesService;

/// Apply a key-value preference to the response object.
pub(crate) fn apply_preference(response: &mut PreferencesResponse, key: &str, value: &str) {
    match key {
        "theme" => response.theme = value.to_string(),
        "subscriptions_view" => response.subscriptions_view = value.to_string(),
        _ => {}
    }
}

impl PreferencesService {
    pub async fn get_preferences(pool: &PgPool, user_id: Uuid) -> AppResult<PreferencesResponse> {
        let prefs = sqlx::query_as::<_, (String, String)>(
            "SELECT key, value FROM user_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut response = PreferencesResponse::default();

        for (key, value) in prefs {
            apply_preference(&mut response, &key, &value);
        }

        Ok(response)
    }

    pub async fn update_preferences(
        pool: &PgPool,
        user_id: Uuid,
        req: UpdatePreferencesRequest,
    ) -> AppResult<PreferencesResponse> {
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
            .await?;
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
            .await?;
        }

        Self::get_preferences(pool, user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_preference_theme() {
        let mut response = PreferencesResponse::default();
        apply_preference(&mut response, "theme", "dark");
        assert_eq!(response.theme, "dark");
        assert_eq!(response.subscriptions_view, "grid"); // unchanged
    }

    #[test]
    fn apply_preference_subscriptions_view() {
        let mut response = PreferencesResponse::default();
        apply_preference(&mut response, "subscriptions_view", "list");
        assert_eq!(response.theme, "system"); // unchanged
        assert_eq!(response.subscriptions_view, "list");
    }

    #[test]
    fn apply_preference_unknown_key_ignored() {
        let mut response = PreferencesResponse::default();
        apply_preference(&mut response, "unknown_key", "value");
        assert_eq!(response.theme, "system");
        assert_eq!(response.subscriptions_view, "grid");
    }

    #[test]
    fn apply_preference_both() {
        let mut response = PreferencesResponse::default();
        apply_preference(&mut response, "theme", "dark");
        apply_preference(&mut response, "subscriptions_view", "table");
        assert_eq!(response.theme, "dark");
        assert_eq!(response.subscriptions_view, "table");
    }

    #[test]
    fn apply_preference_overwrite() {
        let mut response = PreferencesResponse::default();
        apply_preference(&mut response, "theme", "dark");
        apply_preference(&mut response, "theme", "light");
        assert_eq!(response.theme, "light");
    }
}