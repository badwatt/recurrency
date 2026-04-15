use crate::api::client::{api_get, ApiError};
use crate::models::logo::*;

pub async fn search(query: &str) -> Result<LogoSearchResponse, ApiError> {
    api_get(&format!("/api/logos/search?q={}", query)).await
}