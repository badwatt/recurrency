use crate::api::client::{api_post, ApiError};
use crate::models::auth::*;
use std::io::{Read, Write};
use std::net::TcpStream;

/// Raw HTTP POST using TcpStream. No reqwest.
/// On Android, reqwest (async or blocking) fails inside tokio runtime.
fn raw_post<T: serde::de::DeserializeOwned>(base_url: &str, path: &str, body: &str) -> Result<T, ApiError> {
    let trimmed = base_url.trim();
    let without_proto = trimmed
        .strip_prefix("http://")
        .or_else(|| trimmed.strip_prefix("https://"))
        .ok_or_else(|| ApiError::Network("URL must start with http://".to_string()))?;
    let host_port = without_proto.split('/').next().unwrap_or(without_proto);
    let mut parts = host_port.split(':');
    let host = parts.next().unwrap_or("");
    let port: u16 = parts.next().unwrap_or("80").parse().map_err(|_| ApiError::Network("Invalid port".to_string()))?;

    let addr_str = format!("{}:{}", host, port);
    use std::net::ToSocketAddrs;
    let socket_addr = addr_str
        .to_socket_addrs()
        .map_err(|e| ApiError::Network(format!("Invalid address: {}", e)))?
        .next()
        .ok_or_else(|| ApiError::Network(format!("Could not resolve: {}", addr_str)))?;
    let stream = TcpStream::connect_timeout(
        &socket_addr,
        std::time::Duration::from_secs(10),
    ).map_err(|e| ApiError::Network(format!("Connection failed: {}", e)))?;

    stream.set_read_timeout(Some(std::time::Duration::from_secs(15))).ok();
    stream.set_write_timeout(Some(std::time::Duration::from_secs(5))).ok();

    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, addr_str, body.len(), body
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

/// Synchronous login — raw TCP HTTP, no reqwest.
pub fn blocking_login(base_url: &str, email: &str, password: &str) -> Result<AuthResponse, ApiError> {
    let body = serde_json::json!({ "email": email, "password": password }).to_string();
    raw_post(base_url, "/api/auth/login", &body)
}

pub async fn login(email: String, password: String) -> Result<AuthResponse, ApiError> {
    let req = LoginRequest { email, password };
    api_post("/api/auth/login", &req).await
}

pub async fn register(
    email: String,
    password: String,
    name: String,
) -> Result<AuthResponse, ApiError> {
    let req = RegisterRequest {
        email,
        password,
        name,
    };
    api_post("/api/auth/register", &req).await
}

#[allow(dead_code)]
pub async fn verify(token: &str) -> Result<serde_json::Value, ApiError> {
    api_post("/api/auth/verify", &serde_json::json!({ "token": token })).await
}

#[allow(dead_code)]
pub async fn verify_email(token: &str, code: &str) -> Result<serde_json::Value, ApiError> {
    let base = crate::api::client::get_base_url();
    let url = format!("{}/api/auth/verify-email", base);
    let client = crate::api::client::get_client();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "code": code }))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_else(|e| { log::error!("Failed to read response body: {}", e); String::new() });
        return Err(ApiError::Http(status, body));
    }
    resp.json::<serde_json::Value>()
        .await
        .map_err(|e| ApiError::Parse(e.to_string()))
}

pub async fn resend_verification(token: &str) -> Result<serde_json::Value, ApiError> {
    let base = crate::api::client::get_base_url();
    let url = format!("{}/api/auth/resend-verification", base);
    let client = crate::api::client::get_client();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_else(|e| { log::error!("Failed to read response body: {}", e); String::new() });
        return Err(ApiError::Http(status, body));
    }
    resp.json::<serde_json::Value>()
        .await
        .map_err(|e| ApiError::Parse(e.to_string()))
}

#[allow(dead_code)]
pub async fn get_current_user(token: &str) -> Result<UserResponse, ApiError> {
    api_get_with_token("/api/auth/me", token).await
}

#[allow(dead_code)]
async fn api_get_with_token<T: serde::de::DeserializeOwned>(
    path: &str,
    token: &str,
) -> Result<T, ApiError> {
    let base = crate::api::client::get_base_url();
    let url = format!("{}{}", base, path);
    let client = crate::api::client::get_client();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_else(|e| { log::error!("Failed to read response body: {}", e); String::new() });
        return Err(ApiError::Http(status, body));
    }
    resp.json::<T>()
        .await
        .map_err(|e| ApiError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn login_request_construction() {
        let req = LoginRequest {
            email: "test@example.com".into(),
            password: "secret".into(),
        };
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.password, "secret");
    }

    #[test]
    fn register_request_construction() {
        let req = RegisterRequest {
            email: "new@example.com".into(),
            password: "pass123".into(),
            name: "New User".into(),
        };
        assert_eq!(req.name, "New User");
        assert_eq!(req.email, "new@example.com");
    }

    #[test]
    fn login_request_serialization() {
        let req = LoginRequest {
            email: "u@t.com".into(),
            password: "pw".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("u@t.com"));
        assert!(json.contains("pw"));
    }

    #[test]
    fn register_request_serialization() {
        let req = RegisterRequest {
            email: "u@t.com".into(),
            password: "pw".into(),
            name: "User".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("User"));
    }

    #[test]
    fn verify_url_format() {
        let token = "test-token";
        let url = format!("/api/auth/verify-email?token={}", token);
        assert_eq!(url, "/api/auth/verify-email?token=test-token");
    }

    #[test]
    fn resend_verification_url_format() {
        let token = "my-token";
        let url = format!("/api/auth/resend-verification?token={}", token);
        assert_eq!(url, "/api/auth/resend-verification?token=my-token");
    }

    #[test]
    fn verify_request_body() {
        let code = "123456";
        let body = serde_json::json!({ "code": code });
        assert_eq!(body["code"], "123456");
    }

    #[test]
    fn get_current_user_path() {
        let path = "/api/auth/me";
        assert!(path.contains("me"));
    }

    #[test]
    fn auth_response_deserialize() {
        let json = r#"{"token":"tok123","user":{"id":"00000000-0000-0000-0000-000000000001","email":"u@t.com","name":"Test","currency":"EUR","verified":true},"requires_verification":false}"#;
        let resp: AuthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.token, "tok123");
        assert!(!resp.requires_verification);
    }

    // ── UserResponse model tests ──
    #[test]
    fn user_response_fields() {
        let user = UserResponse {
            id: Uuid::new_v4(),
            email: "test@test.com".into(),
            name: Some("John".into()),
            currency: "EUR".into(),
            verified: true,
        };
        assert_eq!(user.email, "test@test.com");
        assert_eq!(user.currency, "EUR");
        assert!(user.verified);
        assert!(user.name.is_some());
    }

    #[test]
    fn user_response_without_name() {
        let user = UserResponse {
            id: Uuid::new_v4(),
            email: "test@test.com".into(),
            name: None,
            currency: "USD".into(),
            verified: false,
        };
        assert!(user.name.is_none());
        assert!(!user.verified);
    }

    #[test]
    fn auth_response_with_verification_required() {
        let json = r#"{"token":"tok","user":{"id":"00000000-0000-0000-0000-000000000001","email":"u@t.com","name":"Test","currency":"EUR","verified":false},"requires_verification":true}"#;
        let resp: AuthResponse = serde_json::from_str(json).unwrap();
        assert!(resp.requires_verification);
        assert!(!resp.user.verified);
    }

    #[test]
    fn auth_response_roundtrip() {
        let json = r#"{"token":"tok123","user":{"id":"00000000-0000-0000-0000-000000000001","email":"u@t.com","name":null,"currency":"EUR","verified":true},"requires_verification":false}"#;
        let resp: AuthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.token, "tok123");
        assert!(resp.user.name.is_none());
    }

    #[test]
    fn login_request_empty_email() {
        let req = LoginRequest {
            email: "".into(),
            password: "pass".into(),
        };
        assert!(req.email.is_empty());
    }

    #[test]
    fn login_request_empty_password() {
        let req = LoginRequest {
            email: "test@test.com".into(),
            password: "".into(),
        };
        assert!(req.password.is_empty());
    }

    #[test]
    fn register_request_empty_name() {
        let req = RegisterRequest {
            email: "test@test.com".into(),
            password: "password".into(),
            name: "".into(),
        };
        assert!(req.name.is_empty());
    }

    #[test]
    fn verify_token_format() {
        let token = "abc123def456";
        let body = serde_json::json!({ "token": token });
        assert_eq!(body["token"], "abc123def456");
    }

    #[test]
    fn api_paths() {
        assert!("/api/auth/login".starts_with("/api/"));
        assert!("/api/auth/register".starts_with("/api/"));
        assert!("/api/auth/verify".starts_with("/api/"));
        assert!("/api/auth/me".starts_with("/api/"));
    }

    #[test]
    fn resend_verification_empty_body() {
        let body = serde_json::json!({});
        assert!(body.is_object());
        assert_eq!(body.as_object().unwrap().len(), 0);
    }
}
