use crate::api::client::{api_get, api_post, ApiError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DemoStatus {
    pub enabled: bool,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct DemoInitResponse {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct DemoResetResponse {
    pub success: bool,
    pub message: String,
}

pub async fn get_status() -> Result<DemoStatus, ApiError> {
    api_get("/api/demo/status").await
}

pub async fn init() -> Result<DemoInitResponse, ApiError> {
    api_post("/api/demo/init", &serde_json::json!({})).await
}

pub async fn reset() -> Result<DemoResetResponse, ApiError> {
    api_post("/api/demo/reset", &serde_json::json!({})).await
}