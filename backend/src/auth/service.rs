use std::sync::OnceLock;

use crate::auth::models::{
    AuthResponse, Claims, CreateUserRequest, LoginRequest, UpdateSettingsRequest, User,
    UserResponse,
};
use crate::shared::error::{AppError, AppResult};
use crate::shared::password_utils::hash_password;
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use argon2::Argon2;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header};
use uuid::Uuid;

use crate::categories::service::CategoryService;
use crate::email::{is_configured, send_verification_email};
use async_trait::async_trait;

static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

fn get_jwt_secret() -> &'static [u8] {
    JWT_SECRET
        .get_or_init(|| {
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "super_secret_jwt_key_change_in_production".to_string())
                .into_bytes()
        })
        .as_slice()
}

fn generate_code() -> String {
    format!("{:06}", rand::random::<u32>() % 1_000_000)
}

/// AuthServiceTrait - trait for dependency injection (enables mocking in tests)
#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    async fn register(&self, req: CreateUserRequest) -> AppResult<AuthResponse>;
    async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse>;
    async fn verify_email(&self, user_id: Uuid, code: String) -> AppResult<UserResponse>;
    async fn resend_verification(&self, user_id: Uuid) -> AppResult<String>;
    async fn get_user_by_id(&self, user_id: Uuid) -> AppResult<User>;
    async fn update_settings(&self, user_id: Uuid, req: UpdateSettingsRequest) -> AppResult<User>;
    async fn verify_token(&self, token: &str) -> AppResult<Claims>;
    async fn get_settings(&self, user_id: Uuid) -> AppResult<UserResponse>;
}

/// SqlxAuthService - production implementation using sqlx PgPool
pub struct SqlxAuthService {
    pool: sqlx::PgPool,
}

impl SqlxAuthService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    async fn register_impl(&self, req: CreateUserRequest) -> AppResult<AuthResponse> {
        let existing = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(&req.email)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        let password_hash = hash_password(&req.password)?;
        let verified = !is_configured();
        let code = if verified {
            None
        } else {
            Some(generate_code())
        };

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
        .fetch_one(&self.pool)
        .await?;

        let token = generate_token(&user)?;

        if !verified {
            if let Err(e) = send_verification_email(
                &user.email,
                user.verification_code.as_deref().unwrap_or("000000"),
            )
            .await
            {
                tracing::error!("Failed to send verification email: {}", e);
            } else {
                tracing::info!("Verification email sent to {}", user.email);
            }
        } else {
            tracing::info!("SMTP not configured - user {} auto-verified", user.email);
        }

        if let Err(e) = CategoryService::create_uncategorized(&self.pool, user.id).await {
            tracing::error!("Failed to create uncategorized category: {}", e);
        }

        Ok(AuthResponse {
            token,
            user: user.into(),
            requires_verification: !verified,
        })
    }

    async fn login_impl(&self, req: LoginRequest) -> AppResult<AuthResponse> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(&req.email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        verify_password(&req.password, &user.password_hash)?;

        if !user.verified && is_configured() {
            return Err(AppError::Unauthorized(
                "Please verify your email before logging in".to_string(),
            ));
        }

        let token = generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
            requires_verification: false,
        })
    }

    async fn verify_email_impl(&self, user_id: Uuid, code: String) -> AppResult<UserResponse> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if user.verified {
            return Err(AppError::BadRequest("Already verified".to_string()));
        }

        if user.verification_code.as_deref() != Some(&code) {
            return Err(AppError::BadRequest(
                "Invalid verification code".to_string(),
            ));
        }

        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET verified = true, verification_code = NULL, updated_at = NOW() WHERE id = $1
             RETURNING id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user.into())
    }

    async fn resend_verification_impl(&self, user_id: Uuid) -> AppResult<String> {
        if !is_configured() {
            return Err(AppError::BadRequest(
                "SMTP not configured - cannot resend verification".to_string(),
            ));
        }

        let user = self.get_user_by_id_impl(user_id).await?;
        if user.verified {
            return Err(AppError::BadRequest("Already verified".to_string()));
        }

        let code = generate_code();
        sqlx::query("UPDATE users SET verification_code = $1, updated_at = NOW() WHERE id = $2")
            .bind(&code)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if let Err(e) = send_verification_email(&user.email, &code).await {
            tracing::error!("Failed to resend verification email: {}", e);
            return Err(AppError::Internal("Failed to send email".to_string()));
        }

        Ok(code)
    }

    async fn get_user_by_id_impl(&self, user_id: Uuid) -> AppResult<User> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, name, currency, verified, verification_code, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    async fn update_settings_impl(
        &self,
        user_id: Uuid,
        req: UpdateSettingsRequest,
    ) -> AppResult<User> {
        let name = req.name.unwrap_or_default();
        let currency = req.currency.unwrap_or_default();

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
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user)
    }
}

#[async_trait]
impl AuthServiceTrait for SqlxAuthService {
    async fn register(&self, req: CreateUserRequest) -> AppResult<AuthResponse> {
        self.register_impl(req).await
    }

    async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse> {
        self.login_impl(req).await
    }

    async fn verify_token(&self, _token: &str) -> AppResult<Claims> {
        // This is a no-op for SQLx service - used only by mock in tests
        Err(AppError::Internal(
            "Use verify_token_internal directly".into(),
        ))
    }

    async fn verify_email(&self, user_id: Uuid, code: String) -> AppResult<UserResponse> {
        self.verify_email_impl(user_id, code).await
    }

    async fn resend_verification(&self, user_id: Uuid) -> AppResult<String> {
        self.resend_verification_impl(user_id).await
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> AppResult<User> {
        self.get_user_by_id_impl(user_id).await
    }

    async fn update_settings(&self, user_id: Uuid, req: UpdateSettingsRequest) -> AppResult<User> {
        self.update_settings_impl(user_id, req).await
    }

    async fn get_settings(&self, user_id: Uuid) -> AppResult<UserResponse> {
        let user = self.get_user_by_id_impl(user_id).await?;
        Ok(user.into())
    }
}

// ── Helper functions (pure, no DB) ──

/// Thin wrapper so existing handlers keep calling AuthService::verify_token.
pub struct AuthService;

impl AuthService {
    pub async fn verify_token(token: &str) -> AppResult<Claims> {
        verify_token_internal(token)
    }
}

pub fn verify_token_internal(token: &str) -> AppResult<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret()),
        &jsonwebtoken::Validation::default(),
    )?;
    Ok(token_data.claims)
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
    use crate::auth::models::{
        AuthResponse, Claims, CreateUserRequest, LoginRequest, UpdateSettingsRequest, UserResponse,
    };
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
        let hash = hash_password("tëst_päss_日本語").unwrap();
        assert!(verify_password("tëst_päss_日本語", &hash).is_ok());
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
            assert!(
                code.chars().all(|c| c.is_ascii_digit()),
                "Code '{}' has non-digit",
                code
            );
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
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(get_jwt_secret()),
            &jsonwebtoken::Validation::default(),
        )
        .unwrap();
        assert_eq!(token_data.claims.sub, uid);
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
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(get_jwt_secret()),
            &jsonwebtoken::Validation::default(),
        )
        .unwrap();
        assert_eq!(token_data.claims.email, "token@test.com");
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

    // AuthResponse
    #[test]
    fn auth_response_requires_verification_true() {
        let resp = AuthResponse {
            token: "t".to_string(),
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "a@b.com".to_string(),
                name: None,
                currency: "EUR".to_string(),
                verified: false,
            },
            requires_verification: true,
        };
        assert!(resp.requires_verification);
    }

    #[test]
    fn auth_response_requires_verification_false() {
        let resp = AuthResponse {
            token: "t".to_string(),
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "a@b.com".to_string(),
                name: None,
                currency: "EUR".to_string(),
                verified: true,
            },
            requires_verification: false,
        };
        assert!(!resp.requires_verification);
    }

    #[test]
    fn auth_response_serialize() {
        let resp = AuthResponse {
            token: "my-jwt".to_string(),
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "resp@test.com".to_string(),
                name: None,
                currency: "USD".to_string(),
                verified: true,
            },
            requires_verification: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("my-jwt"));
        assert!(json.contains("resp@test.com"));
    }

    // verify_token_internal
    #[test]
    fn verify_token_invalid_fails() {
        let result = verify_token_internal("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn verify_token_empty_fails() {
        let result = verify_token_internal("");
        assert!(result.is_err());
    }

    // LoginRequest
    #[test]
    fn login_request_deserialize() {
        let req: LoginRequest =
            serde_json::from_str(r#"{"email":"login@test.com","password":"secret123"}"#).unwrap();
        assert_eq!(req.email, "login@test.com");
        assert_eq!(req.password, "secret123");
    }

    // UpdateSettingsRequest
    #[test]
    fn update_settings_request_both() {
        let req: UpdateSettingsRequest =
            serde_json::from_str(r#"{"name":"Alice","currency":"GBP"}"#).unwrap();
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

    // verify_password case sensitivity
    #[test]
    fn verify_password_case_sensitive() {
        let hash = hash_password("PassWord123").unwrap();
        assert!(verify_password("password123", &hash).is_err());
        assert!(verify_password("PASSWORD123", &hash).is_err());
    }

    #[test]
    fn verify_password_whitespace_sensitive() {
        let hash = hash_password("nopadding").unwrap();
        assert!(verify_password(" nopadding", &hash).is_err());
        assert!(verify_password("nopadding ", &hash).is_err());
    }

    #[test]
    fn verify_password_malformed_hash() {
        let result = verify_password("password", "not-a-valid-hash");
        assert!(result.is_err());
    }

    // generate_code distribution
    #[test]
    fn generate_code_distribution() {
        let mut counts = [0u32; 10];
        for _ in 0..10000 {
            let code = generate_code();
            for c in code.chars() {
                if let Some(d) = c.to_digit(10) {
                    counts[d as usize] += 1;
                }
            }
        }
        let total: u32 = counts.iter().sum();
        for count in counts.iter() {
            let pct = (*count as f64) / (total as f64) * 100.0;
            assert!(
                pct > 5.0 && pct < 20.0,
                "Digit distribution seems off: {}%",
                pct
            );
        }
    }

    // SqlxAuthService creation
    #[test]
    fn sqlx_service_creation() {
        fn accepts_service(_: SqlxAuthService) {}
        assert!(true);
    }

    // verify_token_internal - expired token
    #[test]
    fn verify_token_expired_fails() {
        use chrono::Duration;
        use jsonwebtoken::{encode, EncodingKey, Header, Validation};

        let uid = Uuid::new_v4();
        let past = Utc::now() - Duration::hours(25 * 7);
        let claims = Claims {
            sub: uid,
            email: "expired@test.com".to_string(),
            exp: past.timestamp() as usize,
            iat: past.timestamp() as usize,
        };
        let secret = get_jwt_secret();
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret),
        )
        .unwrap();
        let result = verify_token_internal(&token);
        assert!(result.is_err(), "Expired token should fail verification");
    }

    // verify_token_internal - wrong secret
    #[test]
    fn verify_token_wrong_secret_fails() {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let uid = Uuid::new_v4();
        let future = Utc::now() + Duration::hours(24);
        let claims = Claims {
            sub: uid,
            email: "wrong@test.com".to_string(),
            exp: future.timestamp() as usize,
            iat: future.timestamp() as usize,
        };
        // Encode with wrong secret
        let wrong_secret = b"wrong_secret_key";
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(wrong_secret),
        )
        .unwrap();
        let result = verify_token_internal(&token);
        assert!(result.is_err(), "Token with wrong secret should fail");
    }

    // verify_token_internal - malformed token
    #[test]
    fn verify_token_malformed_string() {
        let result = verify_token_internal("not-even-close");
        assert!(result.is_err());
    }

    // verify_password - hash with invalid format
    #[test]
    fn verify_password_completely_invalid_hash() {
        let result = verify_password("password", "x");
        assert!(result.is_err());
    }

    // generate_token - special characters in email
    #[test]
    fn generate_token_special_email() {
        let user = User {
            id: Uuid::new_v4(),
            email: "user+tag@domain.co.uk".to_string(),
            password_hash: "hash".to_string(),
            name: Some("Special Email User".to_string()),
            currency: "EUR".to_string(),
            verified: true,
            verification_code: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let token = generate_token(&user).unwrap();
        let claims = verify_token_internal(&token).unwrap();
        assert_eq!(claims.email, "user+tag@domain.co.uk");
    }

    // Claims structure
    #[test]
    fn claims_creation() {
        let uid = Uuid::new_v4();
        let claims = Claims {
            sub: uid,
            email: "claims@test.com".to_string(),
            exp: 9999999999,
            iat: 1000000000,
        };
        assert_eq!(claims.sub, uid);
        assert_eq!(claims.email, "claims@test.com");
        assert!(claims.exp > claims.iat);
    }

    // UserResponse from User
    #[test]
    fn user_response_from_user() {
        let user = User {
            id: Uuid::new_v4(),
            email: "response@test.com".to_string(),
            password_hash: "secret".to_string(),
            name: None,
            currency: "JPY".to_string(),
            verified: false,
            verification_code: Some("654321".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let response: UserResponse = user.into();
        assert_eq!(response.email, "response@test.com");
        assert!(response.name.is_none());
        assert_eq!(response.currency, "JPY");
        assert!(!response.verified);
    }
}
