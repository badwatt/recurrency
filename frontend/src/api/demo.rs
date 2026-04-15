use crate::api::client::{api_get, api_post, ApiError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DemoStatus {
    pub initialized: bool,
}

pub async fn get_status() -> Result<DemoStatus, ApiError> {
    api_get("/api/demo/status").await
}

pub async fn init() -> Result<crate::models::auth::AuthResponse, ApiError> {
    api_post("/api/demo/init", &serde_json::json!({})).await
}

pub async fn reset() -> Result<crate::models::auth::AuthResponse, ApiError> {
    api_post("/api/demo/reset", &serde_json::json!({})).await
}