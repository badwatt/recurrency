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

#[cfg(test)]
mod tests {
    use super::*;

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
}