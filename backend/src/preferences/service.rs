use sqlx::PgPool;
use uuid::Uuid;

use crate::preferences::models::{PreferencesResponse, UpdatePreferencesRequest};
use crate::shared::error::AppResult;

pub struct SqlxPreferencesService {
    pub pool: PgPool,
}

impl SqlxPreferencesService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Apply a key-value preference to the response object.
    pub(crate) fn apply_preference(response: &mut PreferencesResponse, key: &str, value: &str) {
        match key {
            "theme" => response.theme = value.to_string(),
            "subscriptions_view" => response.subscriptions_view = value.to_string(),
            "first_day_of_week" => response.first_day_of_week = value.to_string(),
            "date_format" => response.date_format = value.to_string(),
            _ => {}
        }
    }

    pub async fn get_preferences(&self, user_id: Uuid) -> AppResult<PreferencesResponse> {
        let prefs = sqlx::query_as::<_, (String, String)>(
            "SELECT key, value FROM user_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut response = PreferencesResponse::default();

        for (key, value) in prefs {
            SqlxPreferencesService::apply_preference(&mut response, &key, &value);
        }

        Ok(response)
    }

    pub async fn update_preferences(
        &self,
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
            .execute(&self.pool)
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
            .execute(&self.pool)
            .await?;
        }

        if let Some(first_day_of_week) = req.first_day_of_week {
            sqlx::query(
                r#"
                INSERT INTO user_preferences (user_id, key, value)
                VALUES ($1, 'first_day_of_week', $2)
                ON CONFLICT (user_id, key) DO UPDATE SET value = $2
                "#,
            )
            .bind(user_id)
            .bind(&first_day_of_week)
            .execute(&self.pool)
            .await?;
        }

        if let Some(date_format) = req.date_format {
            sqlx::query(
                r#"
                INSERT INTO user_preferences (user_id, key, value)
                VALUES ($1, 'date_format', $2)
                ON CONFLICT (user_id, key) DO UPDATE SET value = $2
                "#,
            )
            .bind(user_id)
            .bind(&date_format)
            .execute(&self.pool)
            .await?;
        }

        self.get_preferences(user_id).await
    }
}

/// Preferences port, injected into `AppState` as `Arc<dyn PreferencesServiceTrait>`.
#[async_trait::async_trait]
pub trait PreferencesServiceTrait: Send + Sync {
    async fn get_preferences(&self, user_id: Uuid) -> AppResult<PreferencesResponse>;
    async fn update_preferences(
        &self,
        user_id: Uuid,
        req: UpdatePreferencesRequest,
    ) -> AppResult<PreferencesResponse>;
}

#[async_trait::async_trait]
impl PreferencesServiceTrait for SqlxPreferencesService {
    async fn get_preferences(&self, user_id: Uuid) -> AppResult<PreferencesResponse> {
        SqlxPreferencesService::get_preferences(self, user_id).await
    }

    async fn update_preferences(
        &self,
        user_id: Uuid,
        req: UpdatePreferencesRequest,
    ) -> AppResult<PreferencesResponse> {
        SqlxPreferencesService::update_preferences(self, user_id, req).await
    }
}

// Backward compatibility: keep PreferencesService for now
pub struct PreferencesService;

impl PreferencesService {
    pub async fn get_preferences(pool: &PgPool, user_id: Uuid) -> AppResult<PreferencesResponse> {
        SqlxPreferencesService::new(pool.clone())
            .get_preferences(user_id)
            .await
    }

    pub async fn update_preferences(
        pool: &PgPool,
        user_id: Uuid,
        req: UpdatePreferencesRequest,
    ) -> AppResult<PreferencesResponse> {
        SqlxPreferencesService::new(pool.clone())
            .update_preferences(user_id, req)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── apply_preference ──

    #[test]
    fn apply_preference_theme() {
        let mut response = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut response, "theme", "dark");
        assert_eq!(response.theme, "dark");
        assert_eq!(response.subscriptions_view, "grid"); // unchanged
    }

    #[test]
    fn apply_preference_subscriptions_view() {
        let mut response = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut response, "subscriptions_view", "list");
        assert_eq!(response.theme, "system"); // unchanged
        assert_eq!(response.subscriptions_view, "list");
    }

    #[test]
    fn apply_preference_unknown_key_ignored() {
        let mut response = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut response, "unknown_key", "value");
        assert_eq!(response.theme, "system");
        assert_eq!(response.subscriptions_view, "grid");
    }

    #[test]
    fn apply_preference_both() {
        let mut response = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut response, "theme", "dark");
        SqlxPreferencesService::apply_preference(&mut response, "subscriptions_view", "table");
        assert_eq!(response.theme, "dark");
        assert_eq!(response.subscriptions_view, "table");
    }

    #[test]
    fn apply_preference_overwrite() {
        let mut response = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut response, "theme", "dark");
        SqlxPreferencesService::apply_preference(&mut response, "theme", "light");
        assert_eq!(response.theme, "light");
    }

    // ── PreferencesResponse defaults ──

    #[test]
    fn preferences_response_default_values() {
        let resp = PreferencesResponse::default();
        assert_eq!(resp.theme, "system");
        assert_eq!(resp.subscriptions_view, "grid");
    }

    #[test]
    fn preferences_response_serialize() {
        let resp = PreferencesResponse {
            theme: "dark".to_string(),
            subscriptions_view: "list".to_string(),
            first_day_of_week: "monday".to_string(),
            date_format: "DD-MM-YYYY".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("dark"));
        assert!(json.contains("list"));
    }

    #[test]
    fn preferences_response_deserialize() {
        let json = r#"{"theme":"dark","subscriptions_view":"compact","first_day_of_week":"sunday","date_format":"YYYY-MM-DD"}"#;
        let resp: PreferencesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.theme, "dark");
        assert_eq!(resp.subscriptions_view, "compact");
        assert_eq!(resp.first_day_of_week, "sunday");
        assert_eq!(resp.date_format, "YYYY-MM-DD");
    }

    // ── UpdatePreferencesRequest deserialization ──

    #[test]
    fn update_preferences_request_both_fields() {
        let req = UpdatePreferencesRequest {
            theme: Some("dark".to_string()),
            subscriptions_view: Some("table".to_string()),
            first_day_of_week: None,
            date_format: None,
        };
        assert!(req.theme.is_some());
        assert!(req.subscriptions_view.is_some());
    }

    #[test]
    fn update_preferences_request_theme_only() {
        let req: UpdatePreferencesRequest = serde_json::from_str(r#"{"theme":"light"}"#).unwrap();
        assert_eq!(req.theme.as_deref(), Some("light"));
        assert!(req.subscriptions_view.is_none());
    }

    #[test]
    fn update_preferences_request_subscriptions_view_only() {
        let req: UpdatePreferencesRequest =
            serde_json::from_str(r#"{"subscriptions_view":"compact"}"#).unwrap();
        assert!(req.theme.is_none());
        assert_eq!(req.subscriptions_view.as_deref(), Some("compact"));
    }

    #[test]
    fn update_preferences_request_empty() {
        let req: UpdatePreferencesRequest = serde_json::from_str("{}").unwrap();
        assert!(req.theme.is_none());
        assert!(req.subscriptions_view.is_none());
    }

    #[test]
    fn preferences_response_clone_preserves() {
        let resp = PreferencesResponse::default();
        let cloned = resp.clone();
        assert_eq!(cloned.theme, resp.theme);
        assert_eq!(cloned.subscriptions_view, resp.subscriptions_view);
    }

    // ── More apply_preference edge cases ──

    #[test]
    fn apply_preference_theme_case_sensitive() {
        let mut response = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut response, "Theme", "dark"); // Wrong case
        assert_eq!(response.theme, "system"); // Should not change
    }

    #[test]
    fn apply_preference_subscriptions_view_case_sensitive() {
        let mut response = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut response, "SUBSCRIPTIONS_VIEW", "list"); // Wrong case
        assert_eq!(response.subscriptions_view, "grid"); // Should not change
    }

    #[test]
    fn apply_preference_empty_string_values() {
        let mut response = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut response, "theme", "");
        assert_eq!(response.theme, "");
    }

    #[test]
    fn apply_preference_unicode_values() {
        let mut response = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut response, "theme", "ダーク");
        assert_eq!(response.theme, "ダーク");
    }

    #[test]
    fn apply_preference_multiple_unknown_keys() {
        let mut response = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut response, "lang", "en");
        SqlxPreferencesService::apply_preference(&mut response, "region", "US");
        SqlxPreferencesService::apply_preference(&mut response, "timezone", "UTC");
        // All should be ignored
        assert_eq!(response.theme, "system");
        assert_eq!(response.subscriptions_view, "grid");
    }

    // ── More PreferencesResponse edge cases ──

    #[test]
    fn preferences_response_with_empty_strings() {
        let resp = PreferencesResponse {
            theme: "".to_string(),
            subscriptions_view: "".to_string(),
            first_day_of_week: "monday".to_string(),
            date_format: "DD-MM-YYYY".to_string(),
        };
        assert_eq!(resp.theme, "");
        assert_eq!(resp.subscriptions_view, "");
    }

    #[test]
    fn preferences_response_with_unicode() {
        let resp = PreferencesResponse {
            theme: "テーマ".to_string(),
            subscriptions_view: "列表".to_string(),
            first_day_of_week: "monday".to_string(),
            date_format: "DD-MM-YYYY".to_string(),
        };
        assert_eq!(resp.theme, "テーマ");
        assert_eq!(resp.subscriptions_view, "列表");
    }

    #[test]
    fn preferences_response_roundtrip() {
        let resp = PreferencesResponse {
            theme: "midnight".to_string(),
            subscriptions_view: "compact".to_string(),
            first_day_of_week: "sunday".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: PreferencesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.theme, resp.theme);
        assert_eq!(parsed.subscriptions_view, resp.subscriptions_view);
    }

    #[test]
    fn preferences_response_complete_deserialize() {
        // PreferencesResponse requires all fields
        let json = r#"{"theme":"light","subscriptions_view":"list","first_day_of_week":"monday","date_format":"DD-MM-YYYY"}"#;
        let resp: PreferencesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.theme, "light");
        assert_eq!(resp.subscriptions_view, "list");
        assert_eq!(resp.date_format, "DD-MM-YYYY");
    }

    // ── UpdatePreferencesRequest edge cases ──

    #[test]
    fn update_preferences_request_both_empty_strings() {
        let json = r#"{"theme":"","subscriptions_view":"","first_day_of_week":""}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert!(req.theme.is_some()); // Empty string is still Some
        assert!(req.subscriptions_view.is_some());
        assert!(req.first_day_of_week.is_some());
    }

    #[test]
    fn update_preferences_request_with_unicode() {
        let json = r#"{"theme":"テーマ","subscriptions_view":"视图","first_day_of_week":"monday"}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.theme.as_deref(), Some("テーマ"));
        assert_eq!(req.subscriptions_view.as_deref(), Some("视图"));
        assert_eq!(req.first_day_of_week.as_deref(), Some("monday"));
    }

    #[test]
    fn update_preferences_request_order_independent() {
        // subscriptions_view first
        let req1: UpdatePreferencesRequest =
            serde_json::from_str(r#"{"subscriptions_view":"list","theme":"dark"}"#).unwrap();
        // theme first
        let req2: UpdatePreferencesRequest =
            serde_json::from_str(r#"{"theme":"dark","subscriptions_view":"list"}"#).unwrap();
        assert_eq!(req1.theme, req2.theme);
        assert_eq!(req1.subscriptions_view, req2.subscriptions_view);
    }

    // ── PreferencesService struct ──

    #[test]
    fn preferences_service_can_be_referenced() {
        // Just test the struct exists and can be referenced
        let _service: PreferencesService = PreferencesService;
    }

    // Apply preference edge cases using the 3-arg function
    #[test]
    fn apply_preference_theme_only() {
        let mut prefs = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut prefs, "theme", "dark");
        assert_eq!(prefs.theme, "dark");
        assert_eq!(prefs.subscriptions_view, "grid"); // unchanged
        assert_eq!(prefs.first_day_of_week, "monday"); // unchanged
    }

    #[test]
    fn apply_preference_view_only() {
        let mut prefs = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut prefs, "subscriptions_view", "list");
        assert_eq!(prefs.theme, "system"); // unchanged
        assert_eq!(prefs.subscriptions_view, "list");
        assert_eq!(prefs.first_day_of_week, "monday"); // unchanged
    }

    #[test]
    fn apply_preference_first_day_of_week() {
        let mut prefs = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut prefs, "first_day_of_week", "sunday");
        assert_eq!(prefs.first_day_of_week, "sunday");
    }

    #[test]
    fn apply_preference_date_format() {
        let mut prefs = PreferencesResponse::default();
        SqlxPreferencesService::apply_preference(&mut prefs, "date_format", "YYYY-MM-DD");
        assert_eq!(prefs.date_format, "YYYY-MM-DD");
    }

    #[test]
    fn apply_preference_both_fields() {
        let mut prefs = PreferencesResponse {
            theme: "light".to_string(),
            subscriptions_view: "grid".to_string(),
            first_day_of_week: "monday".to_string(),
            date_format: "DD-MM-YYYY".to_string(),
        };
        SqlxPreferencesService::apply_preference(&mut prefs, "theme", "midnight");
        SqlxPreferencesService::apply_preference(&mut prefs, "subscriptions_view", "compact");
        assert_eq!(prefs.theme, "midnight");
        assert_eq!(prefs.subscriptions_view, "compact");
    }

    #[test]
    fn apply_preference_no_fields() {
        let prefs = PreferencesResponse {
            theme: "light".to_string(),
            subscriptions_view: "grid".to_string(),
            first_day_of_week: "monday".to_string(),
            date_format: "DD-MM-YYYY".to_string(),
        };
        // No fields to update - nothing changes
        assert_eq!(prefs.theme, "light");
        assert_eq!(prefs.subscriptions_view, "grid");
    }

    // Theme variations
    #[test]
    fn theme_variations() {
        for theme in &["light", "dark", "midnight", "solarized", "nord", "dracula"] {
            let req: UpdatePreferencesRequest =
                serde_json::from_str(&format!(r#"{{"theme":"{}"}}"#, theme)).unwrap();
            assert_eq!(req.theme.as_deref(), Some(*theme));
        }
    }

    // View variations
    #[test]
    fn view_variations() {
        for view in &["grid", "list", "compact", "cards"] {
            let req: UpdatePreferencesRequest =
                serde_json::from_str(&format!(r#"{{"subscriptions_view":"{}"}}"#, view)).unwrap();
            assert_eq!(req.subscriptions_view.as_deref(), Some(*view));
        }
    }
}
