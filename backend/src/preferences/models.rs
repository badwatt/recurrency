use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesResponse {
    pub theme: String,
    pub subscriptions_view: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub theme: Option<String>,
    pub subscriptions_view: Option<String>,
}

impl Default for PreferencesResponse {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            subscriptions_view: "grid".to_string(),
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
    }

    #[test]
    fn update_preferences_request_all_none() {
        let req = UpdatePreferencesRequest {
            theme: None,
            subscriptions_view: None,
        };
        assert!(req.theme.is_none());
        assert!(req.subscriptions_view.is_none());
    }

    #[test]
    fn update_preferences_request_some_values() {
        let req = UpdatePreferencesRequest {
            theme: Some("dark".to_string()),
            subscriptions_view: Some("list".to_string()),
        };
        assert_eq!(req.theme.as_deref(), Some("dark"));
        assert_eq!(req.subscriptions_view.as_deref(), Some("list"));
    }

    #[test]
    fn preferences_response_serde_roundtrip() {
        let prefs = PreferencesResponse {
            theme: "dark".to_string(),
            subscriptions_view: "list".to_string(),
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let deserialized: PreferencesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.theme, "dark");
        assert_eq!(deserialized.subscriptions_view, "list");
    }

    #[test]
    fn preferences_response_json_keys() {
        let prefs = PreferencesResponse::default();
        let json = serde_json::to_value(&prefs).unwrap();
        assert!(json.get("theme").is_some());
        assert!(json.get("subscriptions_view").is_some());
    }
}
