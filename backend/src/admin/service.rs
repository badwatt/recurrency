use std::collections::BTreeMap;

use chrono::Utc;
use serde_json::Value;
use sqlx::PgPool;

use crate::admin::models::{
    AdminSettings, AdminUser, FULL_BACKUP_FORMAT_VERSION, FullBackup, RestoreResult,
};
use crate::shared::constants::DEMO_EMAIL;
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

/// Child tables before parents during a full restore.
const RESTORE_DELETE_ORDER: [&str; 14] = [
    "debt_payments",
    "debts",
    "subscriptions",
    "transactions",
    "notifications",
    "notification_preferences",
    "notification_channels",
    "summary_notification_settings",
    "user_preferences",
    "currency_settings",
    "categories",
    "accounts",
    "users",
    "app_settings",
];

/// Parent tables before children when inserting a full restore.
const RESTORE_INSERT_ORDER: [&str; 14] = [
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
    "app_settings",
];

/// User-owned tables deleted before the users row itself. The order respects
/// the account/category foreign keys on subscriptions, transactions and debts.
const BACKUP_USER_TABLES: [&str; 12] = [
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
];

const USER_DELETE_ORDER: [&str; 12] = [
    "debt_payments",
    "debts",
    "subscriptions",
    "transactions",
    "notifications",
    "notification_preferences",
    "notification_channels",
    "summary_notification_settings",
    "user_preferences",
    "currency_settings",
    "categories",
    "accounts",
];
const REGISTRATION_ENABLED_VALUE: &str = "true";
const REGISTRATION_DISABLED_VALUE: &str = "false";

#[async_trait::async_trait]
pub trait AdminServiceTrait: Send + Sync {
    async fn is_admin(&self, user_id: uuid::Uuid) -> AppResult<bool>;
    async fn list_users(&self) -> AppResult<Vec<AdminUser>>;
    async fn settings(&self) -> AppResult<AdminSettings>;
    async fn set_registration_enabled(&self, enabled: bool) -> AppResult<AdminSettings>;
    async fn full_backup(&self) -> AppResult<FullBackup>;
    async fn restore_backup(&self, backup: FullBackup) -> AppResult<RestoreResult>;
    async fn delete_user(&self, actor_id: uuid::Uuid, user_id: uuid::Uuid) -> AppResult<()>;
    async fn set_admin(
        &self,
        actor_id: uuid::Uuid,
        user_id: uuid::Uuid,
        is_admin: bool,
    ) -> AppResult<AdminUser>;
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
             FROM users WHERE email <> $1 ORDER BY created_at ASC, id ASC",
        )
        .bind(DEMO_EMAIL)
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
            let (query, demo_filtered) = if table == "users" {
                (
                    format!(
                        "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM \"{table}\" t WHERE t.email <> $1"
                    ),
                    true,
                )
            } else if BACKUP_USER_TABLES.contains(&table) {
                (
                    format!(
                        "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM \"{table}\" t WHERE t.user_id NOT IN (SELECT id FROM users WHERE email = $1)"
                    ),
                    true,
                )
            } else {
                (
                    format!(
                        "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM \"{table}\" t"
                    ),
                    false,
                )
            };
            let mut request = sqlx::query_scalar(sqlx::AssertSqlSafe(query));
            if demo_filtered {
                request = request.bind(DEMO_EMAIL);
            }
            let rows: Value = request.fetch_one(&mut *transaction).await?;
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

    async fn restore_backup(&self, backup: FullBackup) -> AppResult<RestoreResult> {
        if backup.format_version != FULL_BACKUP_FORMAT_VERSION {
            return Err(crate::shared::error::AppError::BadRequest(format!(
                "Unsupported backup format: {}",
                backup.format_version
            )));
        }
        for table in RESTORE_INSERT_ORDER {
            let rows = backup.tables.get(table).ok_or_else(|| {
                crate::shared::error::AppError::BadRequest(format!(
                    "Backup is missing the {table} table"
                ))
            })?;
            if !rows.is_array() {
                return Err(crate::shared::error::AppError::BadRequest(format!(
                    "Backup table {table} is not an array"
                )));
            }
        }

        let mut transaction = self.pool.begin().await?;
        for table in RESTORE_DELETE_ORDER {
            let query = format!("DELETE FROM \"{table}\"");
            sqlx::query(sqlx::AssertSqlSafe(query))
                .execute(&mut *transaction)
                .await?;
        }
        for table in RESTORE_INSERT_ORDER {
            let rows = backup
                .tables
                .get(table)
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if rows.is_empty() {
                continue;
            }
            let query = format!(
                "INSERT INTO \"{table}\" SELECT * FROM json_populate_recordset(NULL::\"{table}\", $1::json)"
            );
            sqlx::query(sqlx::AssertSqlSafe(query))
                .bind(Value::Array(rows))
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(RestoreResult {
            restored_tables: RESTORE_INSERT_ORDER.len(),
        })
    }

    async fn delete_user(&self, actor_id: uuid::Uuid, user_id: uuid::Uuid) -> AppResult<()> {
        let is_demo: Option<bool> =
            sqlx::query_scalar("SELECT email = $2 FROM users WHERE id = $1")
                .bind(user_id)
                .bind(DEMO_EMAIL)
                .fetch_optional(&self.pool)
                .await?;
        if is_demo == Some(true) {
            return Err(crate::shared::error::AppError::BadRequest(
                "The demo account is managed by demo mode".to_string(),
            ));
        }
        if actor_id == user_id {
            return Err(crate::shared::error::AppError::BadRequest(
                "You cannot delete your own account from the admin panel".to_string(),
            ));
        }
        let target: Option<bool> = sqlx::query_scalar("SELECT is_admin FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        let Some(target_is_admin) = target else {
            return Err(crate::shared::error::AppError::NotFound(
                "User not found".to_string(),
            ));
        };
        if target_is_admin {
            let admins: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = true")
                    .fetch_one(&self.pool)
                    .await?;
            if admins <= 1 {
                return Err(crate::shared::error::AppError::Conflict(
                    "The last administrator cannot be deleted".to_string(),
                ));
            }
        }

        delete_user_data(&self.pool, user_id).await
    }

    async fn set_admin(
        &self,
        _actor_id: uuid::Uuid,
        user_id: uuid::Uuid,
        is_admin: bool,
    ) -> AppResult<AdminUser> {
        let target = sqlx::query_as::<_, AdminUser>(
            "SELECT id, email, name, currency, verified, is_admin, created_at \
             FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| crate::shared::error::AppError::NotFound("User not found".to_string()))?;
        if target.email == DEMO_EMAIL {
            return Err(crate::shared::error::AppError::BadRequest(
                "The demo account is managed by demo mode".to_string(),
            ));
        }
        if !is_admin && target.is_admin {
            let admins: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = true")
                    .fetch_one(&self.pool)
                    .await?;
            if admins <= 1 {
                return Err(crate::shared::error::AppError::Conflict(
                    "The last administrator cannot be demoted".to_string(),
                ));
            }
        }
        Ok(sqlx::query_as::<_, AdminUser>(
            "UPDATE users SET is_admin = $1, updated_at = NOW() WHERE id = $2 \
             RETURNING id, email, name, currency, verified, is_admin, created_at",
        )
        .bind(is_admin)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?)
    }
}

pub(crate) async fn delete_user_data(
    pool: &PgPool,
    user_id: uuid::Uuid,
) -> AppResult<()> {
    let mut transaction = pool.begin().await?;
    for table in USER_DELETE_ORDER {
        let query = format!("DELETE FROM \"{table}\" WHERE user_id = $1");
        sqlx::query(sqlx::AssertSqlSafe(query))
            .bind(user_id)
            .execute(&mut *transaction)
            .await?;
    }
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(())
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
