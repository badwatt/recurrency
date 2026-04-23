use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesResponse {
    pub theme: String,
    pub subscriptions_view: String,
    pub first_day_of_week: String,
    pub date_format: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub theme: Option<String>,
    pub subscriptions_view: Option<String>,
    pub first_day_of_week: Option<String>,
    pub date_format: Option<String>,
}

impl Default for PreferencesResponse {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            subscriptions_view: "grid".to_string(),
            first_day_of_week: "monday".to_string(),
            date_format: "DD-MM-YYYY".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preferences() {
        let prefs = PreferencesResponse::default();
        assert_eq!(prefs.theme, "system");
        assert_eq!(prefs.subscriptions_view, "grid");
        assert_eq!(prefs.first_day_of_week, "monday");
        assert_eq!(prefs.date_format, "DD-MM-YYYY");
    }

    #[test]
    fn update_preferences_request_all_none() {
        let req = UpdatePreferencesRequest {
            theme: None,
            subscriptions_view: None,
            first_day_of_week: None,
            date_format: None,
        };
        assert!(req.theme.is_none());
        assert!(req.subscriptions_view.is_none());
        assert!(req.first_day_of_week.is_none());
        assert!(req.date_format.is_none());
    }

    #[test]
    fn update_preferences_request_some_values() {
        let req = UpdatePreferencesRequest {
            theme: Some("dark".to_string()),
            subscriptions_view: Some("list".to_string()),
            first_day_of_week: Some("sunday".to_string()),
            date_format: Some("YYYY-MM-DD".to_string()),
        };
        assert_eq!(req.theme.as_deref(), Some("dark"));
        assert_eq!(req.subscriptions_view.as_deref(), Some("list"));
        assert_eq!(req.first_day_of_week.as_deref(), Some("sunday"));
        assert_eq!(req.date_format.as_deref(), Some("YYYY-MM-DD"));
    }

    #[test]
    fn preferences_response_serde_roundtrip() {
        let prefs = PreferencesResponse {
            theme: "dark".to_string(),
            subscriptions_view: "list".to_string(),
            first_day_of_week: "monday".to_string(),
            date_format: "DD-MM-YYYY".to_string(),
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let deserialized: PreferencesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.theme, "dark");
        assert_eq!(deserialized.subscriptions_view, "list");
        assert_eq!(deserialized.first_day_of_week, "monday");
        assert_eq!(deserialized.date_format, "DD-MM-YYYY");
    }

    #[test]
    fn preferences_response_json_keys() {
        let prefs = PreferencesResponse::default();
        let json = serde_json::to_value(&prefs).unwrap();
        assert!(json.get("theme").is_some());
        assert!(json.get("subscriptions_view").is_some());
        assert!(json.get("first_day_of_week").is_some());
        assert!(json.get("date_format").is_some());
    }

    #[test]
    fn preferences_first_day_of_week_valid_values() {
        for day in ["monday", "sunday"] {
            let prefs = PreferencesResponse {
                theme: "system".to_string(),
                subscriptions_view: "grid".to_string(),
                first_day_of_week: day.to_string(),
                date_format: "DD-MM-YYYY".to_string(),
            };
            assert_eq!(prefs.first_day_of_week, day);
        }
    }

    #[test]
    fn preferences_date_format_valid_values() {
        for fmt in ["DD-MM-YYYY", "YYYY-MM-DD"] {
            let prefs = PreferencesResponse {
                theme: "system".to_string(),
                subscriptions_view: "grid".to_string(),
                first_day_of_week: "monday".to_string(),
                date_format: fmt.to_string(),
            };
            assert_eq!(prefs.date_format, fmt);
        }
    }
}
