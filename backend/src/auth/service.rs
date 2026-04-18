use std::sync::OnceLock;

use crate::auth::models::{AuthResponse, Claims, CreateUserRequest, LoginRequest, UpdateSettingsRequest, User, UserResponse, SettingsResponse};
use crate::shared::error::{AppError, AppResult};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use argon2::Argon2;
use crate::shared::password_utils::hash_password;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header};
use sqlx::PgPool;
use uuid::Uuid;

use crate::email::{is_configured, send_verification_email};
use crate::categories::service::CategoryService;

static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

fn get_jwt_secret() -> &'static [u8] {
    JWT_SECRET.get_or_init(|| {
        std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "super_secret_jwt_key_change_in_production".to_string())
            .into_bytes()
    }).as_slice()
}

fn generate_code() -> String {
    format!("{:06}", rand::random::<u32>() % 1_000_000)
}

pub struct AuthService;

impl AuthService {
    pub async fn register(pool: &PgPool, req: CreateUserRequest) -> AppResult<AuthResponse> {
        let existing = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(&req.email)
        .fetch_optional(pool)
        .await?;

        if existing.is_some() {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        let password_hash = hash_password(&req.password)?;
        let verified = !is_configured();
        let code = if verified { None } else { Some(generate_code()) };

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'EUR', $5, $6, $7, $7)
            RETURNING id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.name)
        .bind(verified)
        .bind(code)
        .bind(Utc::now())
        .fetch_one(pool)
        .await?;

        let token = generate_token(&user)?;

        if !verified {
            if let Err(e) = send_verification_email(&user.email, user.verification_code.as_deref().unwrap_or("000000")).await {
                tracing::error!("Failed to send verification email: {}", e);
            } else {
                tracing::info!("Verification email sent to {}", user.email);
            }
        } else {
            tracing::info!("SMTP not configured - user {} auto-verified", user.email);
        }

        if let Err(e) = CategoryService::create_uncategorized(pool, user.id).await {
            tracing::error!("Failed to create uncategorized category: {}", e);
        }

        Ok(AuthResponse {
            token,
            user: user.into(),
            requires_verification: !verified,
        })
    }

    pub async fn login(pool: &PgPool, req: LoginRequest) -> AppResult<AuthResponse> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(&req.email)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        verify_password(&req.password, &user.password_hash)?;

        if !user.verified && is_configured() {
            return Err(AppError::Unauthorized("Please verify your email before logging in".to_string()));
        }

        let token = generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
            requires_verification: false,
        })
    }

    pub async fn verify_token(token: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(get_jwt_secret()),
            &jsonwebtoken::Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    pub async fn verify_email(pool: &PgPool, user_id: Uuid, code: String) -> AppResult<UserResponse> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if user.verified {
            return Err(AppError::BadRequest("Already verified".to_string()));
        }

        if user.verification_code.as_deref() != Some(&code) {
            return Err(AppError::BadRequest("Invalid verification code".to_string()));
        }

        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET verified = true, verification_code = NULL, updated_at = NOW() WHERE id = $1
             RETURNING id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(user.into())
    }

    pub async fn resend_verification(pool: &PgPool, user_id: Uuid) -> AppResult<String> {
        if !is_configured() {
            return Err(AppError::BadRequest("SMTP not configured - cannot resend verification".to_string()));
        }

        let user = Self::get_user_by_id(pool, user_id).await?;
        if user.verified {
            return Err(AppError::BadRequest("Already verified".to_string()));
        }

        let code = generate_code();
        sqlx::query(
            "UPDATE users SET verification_code = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(&code)
        .bind(user_id)
        .execute(pool)
        .await?;

        if let Err(e) = send_verification_email(&user.email, &code).await {
            tracing::error!("Failed to resend verification email: {}", e);
            return Err(AppError::Internal("Failed to send email".to_string()));
        }

        Ok(code)
    }

    pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> AppResult<User> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn update_settings(pool: &PgPool, user_id: Uuid, req: UpdateSettingsRequest) -> AppResult<User> {
        let name = req.name.unwrap_or_default();
        let currency = req.currency.unwrap_or_else(|| String::new());

        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET name = COALESCE(NULLIF($2, ''), name),
                currency = COALESCE(NULLIF($3, ''), currency),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(&name)
        .bind(&currency)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user)
    }
}

fn verify_password(password: &str, hash: &str) -> AppResult<()> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Failed to parse hash: {}", e)))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|e| AppError::Internal(format!("Password verification failed: {:?}", e)))?;

    Ok(())
}

fn generate_token(user: &User) -> AppResult<String> {
    let now = Utc::now();
    let expiration = now + Duration::hours(24 * 7);

    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret()),
    )?;

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    fn make_create_request(email: &str, secret_val: &str, name: &str) -> CreateUserRequest {
        let json = format!(
            r#"{{"email":"{}","password":"{}","name":"{}"}}"#,
            email, secret_val, name
        );
        serde_json::from_str(&json).unwrap()
    }

    // verify_password
    #[test]
    fn verify_password_unicode() {
        let sample_val = "tëst_päss_日本語";
        let hash = hash_password(sample_val).unwrap();
        assert!(verify_password(sample_val, &hash).is_ok());
    }

    #[test]
    fn verify_password_long_secret() {
        let sample_val = "b".repeat(1000);
        let hash = hash_password(&sample_val).unwrap();
        assert!(verify_password(&sample_val, &hash).is_ok());
    }

    #[test]
    fn verify_password_empty_fails() {
        let hash = hash_password("test").unwrap();
        assert!(verify_password("", &hash).is_err());
    }

    #[test]
    fn verify_password_wrong_fails() {
        let hash = hash_password("correct").unwrap();
        assert!(verify_password("wrong", &hash).is_err());
    }

    // generate_code
    #[test]
    fn generate_code_length() {
        let code = generate_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn generate_code_unique() {
        let code1 = generate_code();
        let code2 = generate_code();
        assert_ne!(code1, code2);
    }

    #[test]
    fn generate_code_all_digits() {
        for _ in 0..100 {
            let code = generate_code();
            assert!(code.chars().all(|c| c.is_ascii_digit()), "Code '{}' has non-digit", code);
        }
    }

    #[test]
    fn generate_code_within_range() {
        for _ in 0..100 {
            let code = generate_code();
            let num: u32 = code.parse().unwrap();
            assert!(num <= 999999);
        }
    }

    // generate_token
    #[test]
    fn generate_token_roundtrip() {
        let uid = uuid::Uuid::new_v4();
        let user = User {
            id: uid,
            email: "test@test.com".to_string(),
            password_hash: "hash".to_string(),
            name: Some("Test".to_string()),
            currency: "USD".to_string(),
            verified: true,
            verification_code: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let token = generate_token(&user).unwrap();
        let claims = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(get_jwt_secret()),
            &jsonwebtoken::Validation::default(),
        ).unwrap();
        assert_eq!(claims.claims.sub, uid);
    }

    #[test]
    fn generate_token_contains_email() {
        let user = User {
            id: Uuid::new_v4(),
            email: "token@test.com".to_string(),
            password_hash: "hash".to_string(),
            name: None,
            currency: "EUR".to_string(),
            verified: true,
            verification_code: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let token = generate_token(&user).unwrap();
        let claims = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(get_jwt_secret()),
            &jsonwebtoken::Validation::default(),
        ).unwrap();
        assert_eq!(claims.claims.email, "token@test.com");
    }

    // get_jwt_secret
    #[test]
    fn get_jwt_secret_returns_value() {
        assert!(!get_jwt_secret().is_empty());
    }

    #[test]
    fn get_jwt_secret_same_on_repeat() {
        assert_eq!(get_jwt_secret(), get_jwt_secret());
    }

    // CreateUserRequest validation
    #[test]
    fn create_user_request_valid() {
        let req = make_create_request("test@example.com", "longenough", "Alice");
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_user_request_invalid_email() {
        let req = make_create_request("not-an-email", "longenough", "Alice");
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_user_request_short_secret() {
        let req = make_create_request("test@example.com", "short", "Alice");
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_user_request_empty_name() {
        let req = make_create_request("test@example.com", "longenough", "");
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_user_request_unicode_name() {
        let req = make_create_request("test@test.com", "longenough", "田中太郎");
        assert!(req.validate().is_ok());
    }

    // update_settings field resolution
    #[test]
    fn resolve_name_none() {
        let name: Option<String> = None;
        assert_eq!(name.unwrap_or_default(), "");
    }

    #[test]
    fn resolve_name_some() {
        let name: Option<String> = Some("Alice".to_string());
        assert_eq!(name.unwrap_or_default(), "Alice");
    }

    #[test]
    fn resolve_currency_none() {
        let currency: Option<String> = None;
        assert_eq!(currency.unwrap_or_else(|| String::new()), "");
    }

    #[test]
    fn resolve_currency_some() {
        let currency: Option<String> = Some("EUR".to_string());
        assert_eq!(currency.unwrap_or_else(|| String::new()), "EUR");
    }

    // AuthResponse
    #[test]
    fn auth_response_requires_verification_true() {
        let resp = AuthResponse {
            token: "t".to_string(),
            user: UserResponse { id: Uuid::new_v4(), email: "a@b.com".to_string(), name: None, currency: "EUR".to_string(), verified: false },
            requires_verification: true,
        };
        assert!(resp.requires_verification);
    }

    #[test]
    fn auth_response_requires_verification_false() {
        let resp = AuthResponse {
            token: "t".to_string(),
            user: UserResponse { id: Uuid::new_v4(), email: "a@b.com".to_string(), name: None, currency: "EUR".to_string(), verified: true },
            requires_verification: false,
        };
        assert!(!resp.requires_verification);
    }

    #[test]
    fn auth_response_serialize() {
        let resp = AuthResponse {
            token: "my-jwt".to_string(),
            user: UserResponse { id: Uuid::new_v4(), email: "resp@test.com".to_string(), name: None, currency: "USD".to_string(), verified: true },
            requires_verification: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("my-jwt"));
        assert!(json.contains("resp@test.com"));
    }

    // UserResponse from User
    #[test]
    fn user_response_from_user() {
        let user = User {
            id: Uuid::new_v4(), email: "conv@test.com".to_string(), password_hash: "h".to_string(),
            name: Some("Test User".to_string()), currency: "GBP".to_string(),
            verified: true, verification_code: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        };
        let resp: UserResponse = user.into();
        assert_eq!(resp.email, "conv@test.com");
        assert_eq!(resp.name.as_deref(), Some("Test User"));
        assert_eq!(resp.currency, "GBP");
        assert!(resp.verified);
    }

    #[test]
    fn user_response_no_name() {
        let user = User {
            id: Uuid::new_v4(), email: "noname@test.com".to_string(), password_hash: "h".to_string(),
            name: None, currency: "CAD".to_string(),
            verified: false, verification_code: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        };
        let resp: UserResponse = user.into();
        assert!(resp.name.is_none());
        assert!(!resp.verified);
    }

    // SettingsResponse
    #[test]
    fn settings_response_from_user() {
        let user = User {
            id: Uuid::new_v4(), email: "settings@test.com".to_string(), password_hash: "h".to_string(),
            name: Some("S".to_string()), currency: "CAD".to_string(),
            verified: true, verification_code: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        };
        let resp: SettingsResponse = user.into();
        assert_eq!(resp.user.email, "settings@test.com");
    }

    // Claims
    #[test]
    fn claims_struct_fields() {
        let claims = Claims { sub: Uuid::new_v4(), email: "c@test.com".to_string(), exp: 1234567890, iat: 1234500000 };
        assert_eq!(claims.email, "c@test.com");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn claims_serde_roundtrip() {
        let claims = Claims { sub: Uuid::new_v4(), email: "serde@test.com".to_string(), exp: 9999999999, iat: 1000000000 };
        let json = serde_json::to_string(&claims).unwrap();
        let roundtrip: Claims = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.email, claims.email);
    }

    // AppError
    #[test]
    fn app_error_conflict() {
        let err = AppError::Conflict("Email taken".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Email taken") || msg.contains("Conflict"));
    }

    #[test]
    fn app_error_unauthorized() {
        let err = AppError::Unauthorized("Bad creds".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Bad creds") || msg.contains("Unauthorized"));
    }

    #[test]
    fn app_error_not_found() {
        let err = AppError::NotFound("User not found".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("User not found") || msg.contains("NotFound"));
    }

    #[test]
    fn app_error_internal() {
        let err = AppError::Internal("Something broke".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Something broke") || msg.contains("Internal"));
    }

    // LoginRequest
    #[test]
    fn login_request_deserialize() {
        let req: LoginRequest = serde_json::from_str(r#"{"email":"login@test.com","password":"secret123"}"#).unwrap();
        assert_eq!(req.email, "login@test.com");
        assert_eq!(req.password, "secret123");
    }

    // UpdateSettingsRequest
    #[test]
    fn update_settings_request_both() {
        let req: UpdateSettingsRequest = serde_json::from_str(r#"{"name":"Alice","currency":"GBP"}"#).unwrap();
        assert_eq!(req.name.as_deref(), Some("Alice"));
        assert_eq!(req.currency.as_deref(), Some("GBP"));
    }

    #[test]
    fn update_settings_request_name_only() {
        let req: UpdateSettingsRequest = serde_json::from_str(r#"{"name":"Bob"}"#).unwrap();
        assert!(req.name.is_some());
        assert!(req.currency.is_none());
    }

    #[test]
    fn update_settings_request_empty() {
        let req: UpdateSettingsRequest = serde_json::from_str("{}").unwrap();
        assert!(req.name.is_none());
        assert!(req.currency.is_none());
    }

    // User password_hash skip
    #[test]
    fn user_password_hash_not_in_json() {
        let user = User {
            id: Uuid::new_v4(), email: "user@test.com".to_string(), password_hash: "secret_hash".to_string(),
            name: Some("User".to_string()), currency: "USD".to_string(),
            verified: true, verification_code: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(!json.contains("secret_hash"));
    }
}
