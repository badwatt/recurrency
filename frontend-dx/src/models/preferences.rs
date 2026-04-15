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
}