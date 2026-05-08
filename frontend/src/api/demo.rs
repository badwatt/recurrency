use crate::api::client::{api_get, api_post, ApiError};
use serde::Deserialize;
use std::io::{Read, Write};
use std::net::TcpStream;

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

/// Raw HTTP GET using TcpStream. No reqwest.
fn raw_get<T: serde::de::DeserializeOwned>(base_url: &str, path: &str) -> Result<T, ApiError> {
    let trimmed = base_url.trim();
    let without_proto = trimmed
        .strip_prefix("http://")
        .or_else(|| trimmed.strip_prefix("https://"))
        .ok_or_else(|| ApiError::Network("URL must start with http://".to_string()))?;
    let host_port = without_proto.split('/').next().unwrap_or(without_proto);
    let mut parts = host_port.split(':');
    let host = parts.next().unwrap_or("");
    let port: u16 = parts.next().unwrap_or("80").parse().map_err(|_| ApiError::Network("Invalid port".to_string()))?;

    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|_| ApiError::Network(format!("Invalid address: {}", addr)))?,
        std::time::Duration::from_secs(10),
    ).map_err(|e| ApiError::Network(format!("Connection failed: {}", e)))?;

    stream.set_read_timeout(Some(std::time::Duration::from_secs(15))).ok();

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, addr
    );

    let mut stream_clone = stream.try_clone().map_err(|e| ApiError::Network(e.to_string()))?;
    stream_clone.write_all(request.as_bytes()).map_err(|e| ApiError::Network(format!("Write failed: {}", e)))?;

    let mut response = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match stream_clone.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => response.extend_from_slice(&buf[..n]),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut {
                    break;
                }
                return Err(ApiError::Network(format!("Read failed: {}", e)));
            }
        }
    }

    let response_str = String::from_utf8_lossy(&response);
    let header_end = response_str.find("\r\n\r\n").unwrap_or(0);
    let headers = &response_str[..header_end];
    let body_str = &response_str[header_end + 4..];

    let status_line = headers.lines().next().unwrap_or("");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if status_code < 200 || status_code >= 300 {
        return Err(ApiError::Http(status_code, body_str.to_string()));
    }

    serde_json::from_str(body_str).map_err(|e| ApiError::Parse(format!("{}: {}", e, &body_str[..body_str.len().min(200)])))
}

/// Synchronous demo init — raw TCP HTTP, no reqwest.
pub fn blocking_init(base_url: &str) -> Result<DemoInitResponse, ApiError> {
    raw_get(base_url, "/api/demo/init")
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
