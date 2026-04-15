use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
    pub requires_verification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub currency: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateSettingsRequest {
    pub name: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingsResponse {
    pub user: UserResponse,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_response_roundtrip() {
        let resp = AuthResponse {
            token: "jwt-token".into(),
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "test@test.com".into(),
                name: Some("Test".into()),
                currency: "EUR".into(),
                verified: true,
            },
            requires_verification: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: AuthResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, back);
    }

    #[test]
    fn user_response_optional_name() {
        let user = UserResponse {
            id: Uuid::new_v4(),
            email: "a@b.c".into(),
            name: None,
            currency: "EUR".into(),
            verified: false,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("null"));
    }
}