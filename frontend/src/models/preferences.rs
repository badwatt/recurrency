use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreferencesResponse {
    pub theme: String,
    pub subscriptions_view: String,
    pub first_day_of_week: String,
    pub date_format: String, // Kept for API backward compat; derived from locale in UI
}

impl Default for PreferencesResponse {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            subscriptions_view: "grid".into(),
            first_day_of_week: "monday".into(),
            date_format: "DD/MM/YYYY".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdatePreferencesRequest {
    pub theme: Option<String>,
    pub subscriptions_view: Option<String>,
    pub first_day_of_week: Option<String>,
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
        assert_eq!(prefs.date_format, "DD/MM/YYYY");
    }

    #[test]
    fn roundtrip() {
        let prefs = PreferencesResponse {
            theme: "dark".into(),
            subscriptions_view: "list".into(),
            first_day_of_week: "monday".into(),
            date_format: "YYYY/MM/DD".into(),
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let back: PreferencesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(prefs, back);
    }

    #[test]
    fn update_request_all_none() {
        let req = UpdatePreferencesRequest {
            theme: None,
            subscriptions_view: None,
            first_day_of_week: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdatePreferencesRequest = serde_json::from_str(&json).unwrap();
        assert!(back.theme.is_none());
        assert!(back.subscriptions_view.is_none());
        assert!(back.first_day_of_week.is_none());
    }

    #[test]
    fn update_request_partial_theme() {
        let req = UpdatePreferencesRequest {
            theme: Some("dark".into()),
            subscriptions_view: None,
            first_day_of_week: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdatePreferencesRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.theme, Some("dark".into()));
        assert!(back.subscriptions_view.is_none());
        assert!(back.first_day_of_week.is_none());
    }

    #[test]
    fn update_request_view_only() {
        let req = UpdatePreferencesRequest {
            theme: None,
            subscriptions_view: Some("list".into()),
            first_day_of_week: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdatePreferencesRequest = serde_json::from_str(&json).unwrap();
        assert!(back.theme.is_none());
        assert_eq!(back.subscriptions_view, Some("list".into()));
        assert!(back.first_day_of_week.is_none());
    }

    #[test]
    fn update_request_both() {
        let req = UpdatePreferencesRequest {
            theme: Some("light".into()),
            subscriptions_view: Some("grid".into()),
            first_day_of_week: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdatePreferencesRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.theme, Some("light".into()));
        assert_eq!(back.subscriptions_view, Some("grid".into()));
        assert!(back.first_day_of_week.is_none());
    }

    #[test]
    fn preferences_custom_values() {
        let prefs = PreferencesResponse {
            theme: "catppuccin-mocha".into(),
            subscriptions_view: "table".into(),
            first_day_of_week: "sunday".into(),
            date_format: "YYYY/MM/DD".into(),
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let back: PreferencesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.theme, "catppuccin-mocha");
        assert_eq!(back.subscriptions_view, "table");
        assert_eq!(back.date_format, "YYYY/MM/DD");
    }

    #[test]
    fn update_request_all_fields() {
        let req = UpdatePreferencesRequest {
            theme: Some("light".into()),
            subscriptions_view: Some("list".into()),
            first_day_of_week: Some("sunday".into()),
        };
        assert!(req.theme.is_some());
        assert!(req.subscriptions_view.is_some());
        assert!(req.first_day_of_week.is_some());
    }

    #[test]
    fn update_request_first_day_of_week_only() {
        let req = UpdatePreferencesRequest {
            theme: None,
            subscriptions_view: None,
            first_day_of_week: Some("sunday".into()),
        };
        assert!(req.first_day_of_week.is_some());
        assert_eq!(req.first_day_of_week.unwrap(), "sunday");
    }

    #[test]
    fn preferences_response_equality() {
        let a = PreferencesResponse::default();
        let b = PreferencesResponse::default();
        assert_eq!(a, b);
    }

    #[test]
    fn date_format_valid_values() {
        let prefs = PreferencesResponse::default();
        assert!(["DD/MM/YYYY", "YYYY/MM/DD", "MM/DD/YYYY"].contains(&prefs.date_format.as_str()));
    }
}
