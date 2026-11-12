use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RatesResponse {
    pub base: String,
    pub provider: String,
    pub fetched_at: Option<String>,
    pub rates: HashMap<String, Decimal>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CurrencySettingsResponse {
    pub provider: String,
    pub fixer_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateCurrencySettingsRequest {
    pub provider: String,
    #[serde(default)]
    pub fixer_api_key: Option<String>,
    #[serde(default)]
    pub clear_fixer_api_key: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_deserialize() {
        let json = r#"{"provider":"fixer","fixer_configured":true}"#;
        let settings: CurrencySettingsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(settings.provider, "fixer");
        assert!(settings.fixer_configured);
    }

    #[test]
    fn update_settings_serialize() {
        let req = UpdateCurrencySettingsRequest {
            provider: "fixer".into(),
            fixer_api_key: Some("secret".into()),
            clear_fixer_api_key: false,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("fixer_api_key"));
        assert!(json.contains("secret"));
    }

    #[test]
    fn rates_deserialize() {
        let json = r#"{"base":"EUR","provider":"frankfurter","fetched_at":"2026-01-01T00:00:00Z","rates":{"USD":"1.1"}}"#;
        let rates: RatesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(rates.base, "EUR");
        assert_eq!(rates.rates.get("USD"), Some(&Decimal::new(11, 1)));
    }
}
