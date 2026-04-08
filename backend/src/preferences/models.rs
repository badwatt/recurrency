use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
