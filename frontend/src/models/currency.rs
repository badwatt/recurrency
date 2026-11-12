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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rates_deserialize() {
        let json = r#"{"base":"EUR","provider":"frankfurter","fetched_at":"2026-01-01T00:00:00Z","rates":{"USD":"1.1"}}"#;
        let rates: RatesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(rates.base, "EUR");
        assert_eq!(rates.rates.get("USD"), Some(&Decimal::new(11, 1)));
    }
}
