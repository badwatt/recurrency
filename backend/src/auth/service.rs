use std::sync::OnceLock;

use crate::auth::models::{AuthResponse, Claims, CreateUserRequest, LoginRequest, UpdateSettingsRequest, User, UserResponse};
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

        // Send verification email (if SMTP is configured)
        if !verified {
            if let Err(e) = send_verification_email(&user.email, user.verification_code.as_deref().unwrap_or("000000")).await {
                tracing::error!("Failed to send verification email: {}", e);
            } else {
                tracing::info!("Verification email sent to {}", user.email);
            }
        } else {
            tracing::info!("SMTP not configured - user {} auto-verified", user.email);
        }

        // Create uncategorized category for the new user
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

