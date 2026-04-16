use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreferencesResponse {
    pub theme: String,
    pub subscriptions_view: String,
}

impl Default for PreferencesResponse {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            subscriptions_view: "grid".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdatePreferencesRequest {
    pub theme: Option<String>,
    pub subscriptions_view: Option<String>,
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
    fn roundtrip() {
        let prefs = PreferencesResponse {
            theme: "dark".into(),
            subscriptions_view: "list".into(),
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
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdatePreferencesRequest = serde_json::from_str(&json).unwrap();
        assert!(back.theme.is_none());
        assert!(back.subscriptions_view.is_none());
    }

    #[test]
    fn update_request_partial_theme() {
        let req = UpdatePreferencesRequest {
            theme: Some("dark".into()),
            subscriptions_view: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdatePreferencesRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.theme, Some("dark".into()));
        assert!(back.subscriptions_view.is_none());
    }

    #[test]
    fn update_request_view_only() {
        let req = UpdatePreferencesRequest {
            theme: None,
            subscriptions_view: Some("list".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdatePreferencesRequest = serde_json::from_str(&json).unwrap();
        assert!(back.theme.is_none());
        assert_eq!(back.subscriptions_view, Some("list".into()));
    }

    #[test]
    fn update_request_both() {
        let req = UpdatePreferencesRequest {
            theme: Some("light".into()),
            subscriptions_view: Some("grid".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdatePreferencesRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.theme, Some("light".into()));
        assert_eq!(back.subscriptions_view, Some("grid".into()));
    }

    #[test]
    fn preferences_custom_values() {
        let prefs = PreferencesResponse {
            theme: "catppuccin-mocha".into(),
            subscriptions_view: "table".into(),
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let back: PreferencesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.theme, "catppuccin-mocha");
        assert_eq!(back.subscriptions_view, "table");
    }
}