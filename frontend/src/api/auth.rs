use crate::api::client::{api_post, ApiError};
use crate::models::auth::*;

pub async fn login(email: String, password: String) -> Result<AuthResponse, ApiError> {
    let req = LoginRequest { email, password };
    api_post("/api/auth/login", &req).await
}

pub async fn register(email: String, password: String, name: String) -> Result<AuthResponse, ApiError> {
    let req = RegisterRequest { email, password, name };
    api_post("/api/auth/register", &req).await
}

pub async fn verify(token: &str) -> Result<serde_json::Value, ApiError> {
    api_post("/api/auth/verify", &serde_json::json!({ "token": token })).await
}

pub async fn verify_email(token: &str, code: &str) -> Result<serde_json::Value, ApiError> {
    api_post(&format!("/api/auth/verify-email?token={}", token), &serde_json::json!({ "code": code })).await
}

pub async fn resend_verification(token: &str) -> Result<serde_json::Value, ApiError> {
    api_post(&format!("/api/auth/resend-verification?token={}", token), &serde_json::json!({})).await
}

pub async fn get_current_user(token: &str) -> Result<UserResponse, ApiError> {
    api_get_with_token("/api/auth/me", token).await
}

async fn api_get_with_token<T: serde::de::DeserializeOwned>(path: &str, token: &str) -> Result<T, ApiError> {
    let base = crate::api::client::get_base_url();
    let url = format!("{}{}?token={}", base, path, token);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::Http(status, body));
    }
    resp.json::<T>().await.map_err(|e| ApiError::Parse(e.to_string()))
}