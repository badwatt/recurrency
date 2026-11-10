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
    pub avatar: Option<String>,
    pub name: Option<String>,
    pub currency: String,
    pub verified: bool,
}

/// Conservative client-side email check used before saving profile changes.
pub fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.is_empty() || email.contains(char::is_whitespace) || email.matches('@').count() != 1 {
        return false;
    }
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
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
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub name: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeleteAccountRequest {
    pub password: String,
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
                avatar: None,
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
                avatar: None,
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
                avatar: None,
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
    fn email_validation_accepts_normal_address() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("user+tag@sub.example.co.uk"));
    }

    #[test]
    fn email_validation_rejects_invalid_addresses() {
        for email in ["", "user", "user@", "@example.com", "a b@example.com", "a@b"] {
            assert!(!is_valid_email(email), "should reject {email}");
        }
    }

    #[test]
    fn delete_account_request_serializes_password() {
        let req = DeleteAccountRequest {
            password: "secret".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("secret"));
        let back: DeleteAccountRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn update_settings_request_partial() {
        let req = UpdateSettingsRequest {
            email: None,
                avatar: None,
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
                avatar: None,
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
                avatar: None,
            name: None,
            currency: "GBP".into(),
            verified: true,
        };
        assert_eq!(user.currency, "GBP");
    }

    #[test]
    fn update_settings_both_fields() {
        let req = UpdateSettingsRequest {
            email: None,
                avatar: None,
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
                avatar: None,
                name: Some("Name".into()),
                currency: "CAD".into(),
                verified: true,
            },
        };
        assert_eq!(resp.user.email, "x@y.z");
        assert_eq!(resp.user.currency, "CAD");
    }
}
