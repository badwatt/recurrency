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
