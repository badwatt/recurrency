//! Administrative user CRUD for the standalone binary.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::accounts::service::AccountService;
use crate::admin::service::{AdminServiceTrait, SqlxAdminService, delete_user_data};
use crate::auth::models::is_valid_email;
use crate::categories::service::CategoryService;
use crate::cli::{AdminAction, AdminArgs};
use crate::shared::constants::DEMO_EMAIL;
use crate::shared::password_utils::hash_password;

const MIN_PASSWORD_LENGTH: usize = 8;

fn required<'a>(value: Option<&'a str>, flag: &str) -> Result<&'a str, String> {
    value
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{flag} is required"))
}

fn validate_email(email: &str) -> Result<String, String> {
    let email = email.trim().to_lowercase();
    if !is_valid_email(&email) {
        return Err("invalid email".to_string());
    }
    if email == DEMO_EMAIL {
        return Err("the demo email is reserved for demo mode".to_string());
    }
    Ok(email)
}

fn validate_password(password: &str) -> Result<String, String> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "password must be at least {MIN_PASSWORD_LENGTH} characters"
        ));
    }
    Ok(password.to_string())
}

async fn ensure_email_available(
    pool: &PgPool,
    email: &str,
    except: Option<Uuid>,
) -> Result<(), String> {
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE lower(email) = $1 AND ($2::uuid IS NULL OR id <> $2)")
            .bind(email)
            .bind(except)
            .fetch_optional(pool)
            .await
            .map_err(|error| error.to_string())?;
    if existing.is_some() {
        return Err("email already registered".to_string());
    }
    Ok(())
}

async fn count_admins(pool: &PgPool) -> Result<i64, String> {
    sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = true")
        .fetch_one(pool)
        .await
        .map_err(|error| error.to_string())
}

async fn create_user(
    pool: &PgPool,
    args: &AdminArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let email = validate_email(required(args.email.as_deref(), "--email")?)?;
    let password = validate_password(required(args.password.as_deref(), "--password")?)?;
    let name = required(args.name.as_deref(), "--name")?.trim().to_string();
    ensure_email_available(pool, &email, None).await?;

    let has_real_users: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE email <> $1)")
            .bind(DEMO_EMAIL)
            .fetch_one(pool)
            .await?;
    let is_admin = args.is_admin.unwrap_or(false) || !has_real_users;
    let password_hash = hash_password(&password)?;
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, avatar, password_hash, name, currency, verified, is_admin, created_at, updated_at) \
         VALUES ($1, $2, NULL, $3, $4, 'EUR', true, $5, $6, $6)",
    )
    .bind(user_id)
    .bind(&email)
    .bind(&password_hash)
    .bind(&name)
    .bind(is_admin)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    let defaults = async {
        CategoryService::create_uncategorized(pool, user_id).await?;
        AccountService::create_default(pool, user_id).await?;
        Ok::<(), crate::shared::error::AppError>(())
    }
    .await;
    if let Err(error) = defaults {
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await;
        return Err(error.into());
    }

    println!("Created user {email} ({user_id}), admin={is_admin}");
    Ok(())
}

async fn update_user(
    pool: &PgPool,
    args: &AdminArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = args
        .target
        .as_deref()
        .or(args.email.as_deref())
        .ok_or("admin update needs an email")?;
    let target_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE lower(email) = lower($1)")
        .bind(target)
        .fetch_optional(pool)
        .await?;
    let target_id = target_id.ok_or("user not found")?;
    let target_email: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
        .bind(target_id)
        .fetch_one(pool)
        .await?;
    if target_email == DEMO_EMAIL {
        return Err("the demo account is managed by demo mode".into());
    }

    let new_email = match args.new_email.as_deref() {
        Some(email) => {
            let email = validate_email(email)?;
            ensure_email_available(pool, &email, Some(target_id)).await?;
            Some(email)
        }
        None => None,
    };
    let name = match args.name.as_deref() {
        Some(name) if name.trim().is_empty() => return Err("--name cannot be empty".into()),
        Some(name) => Some(name.trim().to_string()),
        None => None,
    };
    let password_hash = match args.password.as_deref() {
        Some(password) => Some(hash_password(&validate_password(password)?)?),
        None => None,
    };
    if let Some(false) = args.is_admin {
        let target_is_admin: bool = sqlx::query_scalar("SELECT is_admin FROM users WHERE id = $1")
            .bind(target_id)
            .fetch_one(pool)
            .await?;
        if target_is_admin && count_admins(pool).await? <= 1 {
            return Err("the last administrator cannot be demoted".into());
        }
    }
    if new_email.is_none() && name.is_none() && password_hash.is_none() && args.is_admin.is_none() {
        return Err("no changes requested; use --new-email, --name, --password or --admin".into());
    }

    let user = sqlx::query(
        "UPDATE users SET \
         email = COALESCE($2, email), \
         name = COALESCE($3, name), \
         password_hash = COALESCE($4, password_hash), \
         is_admin = COALESCE($5, is_admin), \
         updated_at = NOW() \
         WHERE id = $1 \
         RETURNING id, email, is_admin",
    )
    .bind(target_id)
    .bind(new_email)
    .bind(name)
    .bind(password_hash)
    .bind(args.is_admin)
    .fetch_one(pool)
    .await?;
    let id: Uuid = sqlx::Row::get(&user, "id");
    let email: String = sqlx::Row::get(&user, "email");
    let is_admin: bool = sqlx::Row::get(&user, "is_admin");
    println!("Updated user {email} ({id}), admin={is_admin}");
    Ok(())
}

async fn delete_user(pool: &PgPool, args: &AdminArgs) -> Result<(), Box<dyn std::error::Error>> {
    let target = args
        .target
        .as_deref()
        .or(args.email.as_deref())
        .ok_or("admin delete needs an email")?;
    let row: Option<(Uuid, String, bool)> =
        sqlx::query_as("SELECT id, email, is_admin FROM users WHERE lower(email) = lower($1)")
            .bind(target)
            .fetch_optional(pool)
            .await?;
    let (id, email, is_admin) = row.ok_or("user not found")?;
    if email == DEMO_EMAIL {
        return Err("the demo account is managed by demo mode".into());
    }
    if is_admin && count_admins(pool).await? <= 1 {
        return Err("the last administrator cannot be deleted".into());
    }
    delete_user_data(pool, id).await?;
    println!("Deleted user {email} ({id})");
    Ok(())
}

pub async fn run(pool: &PgPool, args: &AdminArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.action {
        AdminAction::List => {
            let service = SqlxAdminService::new(pool.clone());
            let users = service.list_users().await?;
            if users.is_empty() {
                println!("No users.");
            } else {
                for user in users {
                    let role = if user.is_admin { "admin" } else { "user" };
                    println!(
                        "{}\t{}\t{}\tverified={}",
                        user.email,
                        user.name.unwrap_or_default(),
                        role,
                        user.verified
                    );
                }
            }
            Ok(())
        }
        AdminAction::Create => create_user(pool, args).await,
        AdminAction::Update => update_user(pool, args).await,
        AdminAction::Delete => delete_user(pool, args).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_validation_normalizes_and_rejects_demo() {
        assert_eq!(validate_email(" User@Example.COM ").unwrap(), "user@example.com");
        assert!(validate_email(DEMO_EMAIL).is_err());
        assert!(validate_email("invalid").is_err());
    }

    #[test]
    fn password_requires_minimum_length() {
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password("short").is_err());
    }
}
