use crate::api::client::{ApiError, api_get};
use crate::models::currency::RatesResponse;

pub async fn get_rates(base: &str) -> Result<RatesResponse, ApiError> {
    api_get(&format!("/api/currency/rates?base={}", base)).await
}
