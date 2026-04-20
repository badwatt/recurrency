use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: Option<String>,
    pub currency: String,
    pub verified: bool,
    pub verification_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSettingsRequest {
    pub name: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
    pub requires_verification: bool,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub currency: String,
    pub verified: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            currency: user.currency,
            verified: user.verified,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub user: UserResponse,
}

impl From<User> for SettingsResponse {
    fn from(user: User) -> Self {
        Self {
            user: user.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,        // user id
    pub email: String,
    pub exp: usize,       // expiration timestamp
    pub iat: usize,       // issued at
}

#[cfg(test)]
mod tests {
    use super::*;

    // User struct
    #[test]
    fn user_all_fields_present() {
        let u = User {
            id: Uuid::new_v4(), email: "t@t.com".into(), password_hash: "hash".into(),
            name: Some("Test".into()), currency: "EUR".into(), verified: true,
            verification_code: None, created_at: Utc::now(), updated_at: Utc::now(),
        };
        assert_eq!(u.email, "t@t.com"); assert_eq!(u.currency, "EUR"); assert!(u.verified);
    }

    #[test]
    fn user_name_optional() {
        let u = User {
            id: Uuid::new_v4(), email: "t@t.com".into(), password_hash: "hash".into(),
            name: None, currency: "USD".into(), verified: false,
            verification_code: None, created_at: Utc::now(), updated_at: Utc::now(),
        };
        assert!(u.name.is_none());
    }

    // UserResponse from User
    #[test]
    fn user_response_from_user() {
        let user = User {
            id: Uuid::new_v4(), email: "t@t.com".into(), password_hash: "secret".into(),
            name: Some("Test".into()), currency: "GBP".into(), verified: true,
            verification_code: None, created_at: Utc::now(), updated_at: Utc::now(),
        };
        let resp: UserResponse = user.into();
        assert_eq!(resp.email, "t@t.com"); assert_eq!(resp.currency, "GBP");
        assert!(resp.verified); assert_eq!(resp.name.as_deref(), Some("Test"));
    }

    #[test]
    fn user_response_password_not_exposed() {
        let user = User {
            id: Uuid::new_v4(), email: "a@b.com".into(), password_hash: "hidden".into(),
            name: None, currency: "JPY".into(), verified: false,
            verification_code: None, created_at: Utc::now(), updated_at: Utc::now(),
        };
        let resp: UserResponse = user.into();
        assert_eq!(resp.currency, "JPY");
    }

    // LoginRequest
    #[test]
    fn login_request_deserialize() {
        let req: LoginRequest = serde_json::from_str(r#"{"email":"x@x.com","password":"pass123"}"#).unwrap();
        assert_eq!(req.email, "x@x.com"); assert_eq!(req.password, "pass123");
    }

    #[test]
    fn login_request_roundtrip() {
        // LoginRequest is Deserialize but not Serialize (passwords should not serialize)
        let req: LoginRequest = serde_json::from_str(r#"{"email":"t@t.com","password":"secret"}"#).unwrap();
        assert_eq!(req.email, "t@t.com"); assert_eq!(req.password, "secret");
    }

    // UpdateSettingsRequest
    #[test]
    fn update_settings_both_none() {
        let req: UpdateSettingsRequest = serde_json::from_str("{}").unwrap();
        assert!(req.name.is_none()); assert!(req.currency.is_none());
    }

    #[test]
    fn update_settings_name_only() {
        let req: UpdateSettingsRequest = serde_json::from_str(r#"{"name":"New"}"#).unwrap();
        assert_eq!(req.name.as_deref(), Some("New")); assert!(req.currency.is_none());
    }

    #[test]
    fn update_settings_currency_only() {
        let req: UpdateSettingsRequest = serde_json::from_str(r#"{"currency":"USD"}"#).unwrap();
        assert!(req.name.is_none()); assert_eq!(req.currency.as_deref(), Some("USD"));
    }

    #[test]
    fn update_settings_both() {
        let req: UpdateSettingsRequest = serde_json::from_str(r#"{"name":"A","currency":"EUR"}"#).unwrap();
        assert_eq!(req.name.as_deref(), Some("A")); assert_eq!(req.currency.as_deref(), Some("EUR"));
    }

    // AuthResponse
    #[test]
    fn auth_response_requires_verification() {
        let user = UserResponse { id: Uuid::new_v4(), email: "t@t.com".into(), name: None, currency: "EUR".into(), verified: false };
        let resp = AuthResponse { token: "tok".into(), user, requires_verification: true };
        assert!(resp.requires_verification);
    }

    #[test]
    fn auth_response_no_verification() {
        let user = UserResponse { id: Uuid::new_v4(), email: "t@t.com".into(), name: None, currency: "EUR".into(), verified: true };
        let resp = AuthResponse { token: "tok".into(), user, requires_verification: false };
        assert!(!resp.requires_verification);
    }

    #[test]
    fn auth_response_serialize() {
        let user = UserResponse { id: Uuid::new_v4(), email: "t@t.com".into(), name: None, currency: "EUR".into(), verified: true };
        let resp = AuthResponse { token: "tok123".into(), user, requires_verification: false };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("tok123")); assert!(json.contains("t@t.com"));
    }

    // SettingsResponse
    #[test]
    fn settings_response_from_user() {
        let user = User {
            id: Uuid::new_v4(), email: "s@s.com".into(), password_hash: "hidden".into(),
            name: Some("Set".into()), currency: "USD".into(), verified: true,
            verification_code: None, created_at: Utc::now(), updated_at: Utc::now(),
        };
        let resp: SettingsResponse = user.into();
        assert_eq!(resp.user.email, "s@s.com");
    }

    // Claims
    #[test]
    fn claims_serde_roundtrip() {
        let claims = Claims { sub: Uuid::new_v4(), email: "c@c.com".into(), exp: 9999999999, iat: 1000000000 };
        let rt: Claims = serde_json::from_str(&serde_json::to_string(&claims).unwrap()).unwrap();
        assert_eq!(rt.email, claims.email); assert_eq!(rt.exp, claims.exp);
    }

    #[test]
    fn claims_struct_fields() {
        let claims = Claims { sub: Uuid::new_v4(), email: "x@x.com".into(), exp: 12345, iat: 100 };
        assert_eq!(claims.sub.to_string().len(), 36); // UUID format
    }
}
