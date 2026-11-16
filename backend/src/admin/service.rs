use std::collections::BTreeMap;

use chrono::Utc;
use serde_json::Value;
use sqlx::PgPool;

use crate::admin::models::{AdminSettings, AdminUser, FULL_BACKUP_FORMAT_VERSION, FullBackup};
use crate::shared::error::AppResult;

/// Every application table that belongs in a full installation backup.
const BACKUP_TABLES: [&str; 15] = [
    "users",
    "accounts",
    "categories",
    "subscriptions",
    "transactions",
    "debts",
    "debt_payments",
    "notification_preferences",
    "notifications",
    "notification_channels",
    "summary_notification_settings",
    "user_preferences",
    "currency_settings",
    "app_secrets",
    "app_settings",
];

const REGISTRATION_KEY: &str = "registration_enabled";
const REGISTRATION_ENABLED_VALUE: &str = "true";
const REGISTRATION_DISABLED_VALUE: &str = "false";

#[async_trait::async_trait]
pub trait AdminServiceTrait: Send + Sync {
    async fn is_admin(&self, user_id: uuid::Uuid) -> AppResult<bool>;
    async fn list_users(&self) -> AppResult<Vec<AdminUser>>;
    async fn settings(&self) -> AppResult<AdminSettings>;
    async fn set_registration_enabled(&self, enabled: bool) -> AppResult<AdminSettings>;
    async fn full_backup(&self) -> AppResult<FullBackup>;
}

pub struct SqlxAdminService {
    pool: PgPool,
}

impl SqlxAdminService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Registration is enabled by default until an administrator changes it.
pub async fn registration_enabled(pool: &PgPool) -> AppResult<bool> {
    let value: Option<String> = sqlx::query_scalar("SELECT value FROM app_settings WHERE key = $1")
        .bind(REGISTRATION_KEY)
        .fetch_optional(pool)
        .await?;
    Ok(value.as_deref() != Some(REGISTRATION_DISABLED_VALUE))
}

#[async_trait::async_trait]
impl AdminServiceTrait for SqlxAdminService {
    async fn is_admin(&self, user_id: uuid::Uuid) -> AppResult<bool> {
        let is_admin: Option<bool> = sqlx::query_scalar("SELECT is_admin FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(is_admin.unwrap_or(false))
    }

    async fn list_users(&self) -> AppResult<Vec<AdminUser>> {
        Ok(sqlx::query_as::<_, AdminUser>(
            "SELECT id, email, name, currency, verified, is_admin, created_at \
             FROM users ORDER BY created_at ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn settings(&self) -> AppResult<AdminSettings> {
        Ok(AdminSettings {
            registration_enabled: registration_enabled(&self.pool).await?,
        })
    }

    async fn set_registration_enabled(&self, enabled: bool) -> AppResult<AdminSettings> {
        sqlx::query(
            "INSERT INTO app_settings (key, value, updated_at) \
             VALUES ($1, $2, NOW()) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()",
        )
        .bind(REGISTRATION_KEY)
        .bind(if enabled {
            REGISTRATION_ENABLED_VALUE
        } else {
            REGISTRATION_DISABLED_VALUE
        })
        .execute(&self.pool)
        .await?;
        self.settings().await
    }

    async fn full_backup(&self) -> AppResult<FullBackup> {
        let mut transaction = self.pool.begin().await?;
        let mut tables = BTreeMap::new();
        for table in BACKUP_TABLES {
            let query =
                format!("SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM \"{table}\" t");
            let rows: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
                .fetch_one(&mut *transaction)
                .await?;
            tables.insert(table.to_string(), rows);
        }
        transaction.commit().await?;
        Ok(FullBackup {
            format_version: FULL_BACKUP_FORMAT_VERSION,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            exported_at: Utc::now(),
            tables,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_tables_are_unique_and_cover_the_app() {
        let mut seen = std::collections::BTreeSet::new();
        for table in BACKUP_TABLES {
            assert!(seen.insert(table), "duplicate backup table: {table}");
        }
        assert!(seen.contains("users"));
        assert!(seen.contains("app_settings"));
    }

    #[test]
    fn settings_model_is_serializable() {
        let settings = AdminSettings {
            registration_enabled: false,
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("registration_enabled"));
    }

    #[test]
    fn full_backup_is_versioned() {
        assert_eq!(FULL_BACKUP_FORMAT_VERSION, 1);
    }
}
