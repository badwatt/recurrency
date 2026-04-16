use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreferencesResponse {
    pub theme: String,
    pub subscriptions_view: String,
    pub first_day_of_week: String, // "sunday" | "monday", default "monday"
}

impl Default for PreferencesResponse {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            subscriptions_view: "grid".into(),
            first_day_of_week: "monday".into(),
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
    }

    #[test]
    fn roundtrip() {
        let prefs = PreferencesResponse {
            theme: "dark".into(),
            subscriptions_view: "list".into(),
            first_day_of_week: "monday".into(),
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
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let back: PreferencesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.theme, "catppuccin-mocha");
        assert_eq!(back.subscriptions_view, "table");
    }
}