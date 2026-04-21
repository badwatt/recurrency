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

    #[test]
    fn login_request_roundtrip() {
        let req = LoginRequest {
            email: "user@example.com".into(),
            password: "s3cret".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: LoginRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.email, "user@example.com");
        assert_eq!(back.password, "s3cret");
    }

    #[test]
    fn register_request_roundtrip() {
        let req = RegisterRequest {
            email: "new@example.com".into(),
            password: "pass123".into(),
            name: "New User".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: RegisterRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "New User");
    }

    #[test]
    fn settings_response_roundtrip() {
        let resp = SettingsResponse {
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "user@test.com".into(),
                name: Some("Test".into()),
                currency: "USD".into(),
                verified: true,
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: SettingsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back, resp);
    }

    #[test]
    fn update_settings_request_partial() {
        let req = UpdateSettingsRequest {
            name: Some("New Name".into()),
            currency: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdateSettingsRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, Some("New Name".into()));
        assert!(back.currency.is_none());
    }

    #[test]
    fn auth_response_requires_verification() {
        let resp = AuthResponse {
            token: "tok".into(),
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "a@b.c".into(),
                name: None,
                currency: "EUR".into(),
                verified: false,
            },
            requires_verification: true,
        };
        assert!(resp.requires_verification);
        assert!(!resp.user.verified);
    }

    #[test]
    fn login_request_serialization() {
        let req = LoginRequest {
            email: "user@test.com".into(),
            password: "pass123".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("user@test.com"));
        assert!(json.contains("pass123"));
    }

    #[test]
    fn register_request_serialization() {
        let req = RegisterRequest {
            email: "new@test.com".into(),
            password: "secret".into(),
            name: "Test User".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Test User"));
    }

    #[test]
    fn user_response_currency_field() {
        let user = UserResponse {
            id: Uuid::new_v4(),
            email: "u@t.com".into(),
            name: None,
            currency: "GBP".into(),
            verified: true,
        };
        assert_eq!(user.currency, "GBP");
    }

    #[test]
    fn update_settings_both_fields() {
        let req = UpdateSettingsRequest {
            name: Some("New".into()),
            currency: Some("JPY".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdateSettingsRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, Some("New".into()));
        assert_eq!(back.currency, Some("JPY".into()));
    }

    #[test]
    fn settings_response_user_nested() {
        let resp = SettingsResponse {
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "x@y.z".into(),
                name: Some("Name".into()),
                currency: "CAD".into(),
                verified: true,
            },
        };
        assert_eq!(resp.user.email, "x@y.z");
        assert_eq!(resp.user.currency, "CAD");
    }
}
