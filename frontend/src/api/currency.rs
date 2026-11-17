use crate::api::client::{ApiError, api_get, api_put};
use crate::models::currency::{
    CurrencySettingsResponse, RatesResponse, UpdateCurrencySettingsRequest,
};

pub async fn get_rates(base: &str) -> Result<RatesResponse, ApiError> {
    api_get(&format!("/api/currency/rates?base={}", base)).await
}

pub async fn get_settings() -> Result<CurrencySettingsResponse, ApiError> {
    api_get("/api/currency/settings").await
}

pub async fn update_settings(
    req: &UpdateCurrencySettingsRequest,
) -> Result<CurrencySettingsResponse, ApiError> {
    api_put("/api/currency/settings", req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::test_support::with_server;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn currency_endpoints_exchange_json() {
        let rates: RatesResponse = with_server(
            r#"{"base":"EUR","provider":"frankfurter","fetched_at":null,"rates":{"USD":"1.1"}}"#,
            get_rates("EUR"),
        )
        .await
        .unwrap();
        assert_eq!(rates.base, "EUR");

        let settings: CurrencySettingsResponse = with_server(
            r#"{"provider":"fixer","fixer_configured":true}"#,
            get_settings(),
        )
        .await
        .unwrap();
        assert!(settings.fixer_configured);

        let updated: CurrencySettingsResponse = with_server(
            r#"{"provider":"frankfurter","fixer_configured":false}"#,
            update_settings(&UpdateCurrencySettingsRequest {
                provider: "frankfurter".into(),
                fixer_api_key: None,
                clear_fixer_api_key: true,
            }),
        )
        .await
        .unwrap();
        assert_eq!(updated.provider, "frankfurter");
    }
}
