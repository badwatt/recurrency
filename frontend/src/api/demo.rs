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

/// Synchronous demo init for use inside tokio::task::spawn_blocking.
pub fn blocking_init(base_url: &str) -> Result<DemoInitResponse, ApiError> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| ApiError::Network(e.to_string()))?;
    let url = format!("{}/api/demo/init", base_url);
    let resp = client.get(&url).send().map_err(|e| ApiError::Network(e.to_string()))?;
    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(ApiError::Http(status, body));
    }
    resp.json::<DemoInitResponse>().map_err(|e| ApiError::Parse(e.to_string()))
}

pub async fn get_status() -> Result<DemoStatus, ApiError> {
    api_get("/api/demo/status").await
}

pub async fn init() -> Result<DemoInitResponse, ApiError> {
    api_get("/api/demo/init").await
}

pub async fn reset() -> Result<DemoResetResponse, ApiError> {
    api_post("/api/demo/reset", &serde_json::json!({})).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_status_deserialize() {
        let json = r#"{"enabled":true,"email":"demo@test.com"}"#;
        let status: DemoStatus = serde_json::from_str(json).unwrap();
        assert!(status.enabled);
        assert_eq!(status.email, "demo@test.com");
    }

    #[test]
    fn demo_status_disabled() {
        let json = r#"{"enabled":false,"email":""}"#;
        let status: DemoStatus = serde_json::from_str(json).unwrap();
        assert!(!status.enabled);
        assert!(status.email.is_empty());
    }

    #[test]
    fn demo_init_response_deserialize() {
        let json = r#"{"email":"demo@recurrency.com","password":"secret123"}"#;
        let resp: DemoInitResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.email, "demo@recurrency.com");
        assert_eq!(resp.password, "secret123");
    }

    #[test]
    fn demo_reset_response_deserialize() {
        let json = r#"{"success":true,"message":"Demo data reset."}"#;
        let resp: DemoResetResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert_eq!(resp.message, "Demo data reset.");
    }

    #[test]
    fn demo_reset_response_failure() {
        let json = r#"{"success":false,"message":"Demo mode not enabled."}"#;
        let resp: DemoResetResponse = serde_json::from_str(json).unwrap();
        assert!(!resp.success);
        assert_eq!(resp.message, "Demo mode not enabled.");
    }

    #[test]
    fn demo_status_deserialize_roundtrip() {
        let json = r#"{"enabled":true,"email":"test@test.com"}"#;
        let status: DemoStatus = serde_json::from_str(json).unwrap();
        assert!(status.enabled);
        assert_eq!(status.email, "test@test.com");
    }

    #[test]
    fn demo_status_api_path() {
        let path = "/api/demo/status";
        assert!(path.starts_with("/api/demo"));
    }

    #[test]
    fn demo_init_api_path() {
        let path = "/api/demo/init";
        assert!(path.starts_with("/api/demo"));
    }

    #[test]
    fn demo_reset_api_path() {
        let path = "/api/demo/reset";
        assert!(path.starts_with("/api/demo"));
    }
}
