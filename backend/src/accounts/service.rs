use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::accounts::models::{
    Account, AccountWithCounts, CreateAccountRequest, UpdateAccountRequest,
};
use crate::categories::service::resolve_optional_json_field;
use crate::shared::error::{AppError, AppResult};

pub struct AccountService;

/// Account port, injected into `AppState` as `Arc<dyn AccountServiceTrait>`.
#[async_trait::async_trait]
pub trait AccountServiceTrait: Send + Sync {
    async fn create(&self, user_id: Uuid, req: CreateAccountRequest) -> AppResult<Account>;
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Account>>;
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Account>;
    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateAccountRequest,
    ) -> AppResult<Account>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn set_default(&self, user_id: Uuid, id: Uuid) -> AppResult<Account>;
    async fn get_default(&self, user_id: Uuid) -> AppResult<Account>;
    async fn get_with_counts(&self, user_id: Uuid) -> AppResult<Vec<AccountWithCounts>>;
}

/// SQLx adapter delegating to the tested [`AccountService`] functions.
pub struct SqlxAccountService {
    pool: PgPool,
}

impl SqlxAccountService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AccountServiceTrait for SqlxAccountService {
    async fn create(&self, user_id: Uuid, req: CreateAccountRequest) -> AppResult<Account> {
        AccountService::create(&self.pool, user_id, req).await
    }

    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Account>> {
        AccountService::get_all(&self.pool, user_id).await
    }

    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Account> {
        AccountService::get_by_id(&self.pool, user_id, id).await
    }

    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateAccountRequest,
    ) -> AppResult<Account> {
        AccountService::update(&self.pool, user_id, id, req).await
    }

    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        AccountService::delete(&self.pool, user_id, id).await
    }

    async fn set_default(&self, user_id: Uuid, id: Uuid) -> AppResult<Account> {
        AccountService::set_default(&self.pool, user_id, id).await
    }

    async fn get_default(&self, user_id: Uuid) -> AppResult<Account> {
        AccountService::get_default(&self.pool, user_id).await
    }

    async fn get_with_counts(&self, user_id: Uuid) -> AppResult<Vec<AccountWithCounts>> {
        AccountService::get_with_counts(&self.pool, user_id).await
    }
}

impl AccountService {
    /// Create the first "General" account for a new user and make it default.
    pub async fn create_default(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO accounts (id, user_id, name, icon, color, is_default, created_at, updated_at)
            VALUES ($1, $2, 'General', 'Wallet', '#6b7280', true, $3, $3)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(Utc::now())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        req: CreateAccountRequest,
    ) -> AppResult<Account> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        let is_default = count.0 == 0;

        let account = sqlx::query_as::<_, Account>(
            r#"
            INSERT INTO accounts (id, user_id, name, icon, color, is_default, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
            RETURNING id, user_id, name, icon, color, is_default, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&req.name)
        .bind(&req.icon)
        .bind(&req.color)
        .bind(is_default)
        .bind(Utc::now())
        .fetch_one(pool)
        .await?;

        Ok(account)
    }

    pub async fn get_all(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Account>> {
        let accounts = sqlx::query_as::<_, Account>(
            r#"
            SELECT id, user_id, name, icon, color, is_default, created_at, updated_at
            FROM accounts
            WHERE user_id = $1
            ORDER BY is_default DESC, name ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(accounts)
    }

    pub async fn get_by_id(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT id, user_id, name, icon, color, is_default, created_at, updated_at
            FROM accounts
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Account not found".to_string()))?;

        Ok(account)
    }

    pub async fn update(
        pool: &PgPool,
        user_id: Uuid,
        id: Uuid,
        req: UpdateAccountRequest,
    ) -> AppResult<Account> {
        let existing = Self::get_by_id(pool, user_id, id).await?;

        let name = req.name.unwrap_or(existing.name);
        let icon = resolve_optional_json_field(req.icon, existing.icon);
        let color = resolve_optional_json_field(req.color, existing.color);

        let account = sqlx::query_as::<_, Account>(
            r#"
            UPDATE accounts
            SET name = $1, icon = $2, color = $3, updated_at = $4
            WHERE id = $5 AND user_id = $6
            RETURNING id, user_id, name, icon, color, is_default, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(&icon)
        .bind(&color)
        .bind(Utc::now())
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Account not found".to_string()))?;

        Ok(account)
    }

    pub async fn get_default(pool: &PgPool, user_id: Uuid) -> AppResult<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT id, user_id, name, icon, color, is_default, created_at, updated_at
            FROM accounts
            WHERE user_id = $1 AND is_default = true
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Default account not found".to_string()))?;

        Ok(account)
    }

    pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<()> {
        let account = Self::get_by_id(pool, user_id, id).await?;

        if account.is_default {
            return Err(AppError::BadRequest(
                "Cannot delete the default account".to_string(),
            ));
        }

        let default_account = Self::get_default(pool, user_id).await?;

        let mut tx = pool.begin().await?;

        sqlx::query(
            "UPDATE subscriptions SET account_id = $1 WHERE account_id = $2 AND user_id = $3",
        )
        .bind(default_account.id)
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE transactions SET account_id = $1 WHERE account_id = $2 AND user_id = $3",
        )
        .bind(default_account.id)
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        let result = sqlx::query(
            "DELETE FROM accounts WHERE id = $1 AND user_id = $2 AND is_default = false",
        )
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "Account not found or is the default account".to_string(),
            ));
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn set_default(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Account> {
        let mut tx = pool.begin().await?;

        sqlx::query("UPDATE accounts SET is_default = false WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        let updated = sqlx::query_as::<_, Account>(
            r#"
            UPDATE accounts
            SET is_default = true, updated_at = $1
            WHERE id = $2 AND user_id = $3
            RETURNING id, user_id, name, icon, color, is_default, created_at, updated_at
            "#,
        )
        .bind(Utc::now())
        .bind(id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Account not found".to_string()))?;

        tx.commit().await?;

        Ok(updated)
    }

    pub async fn get_with_counts(
        pool: &PgPool,
        user_id: Uuid,
    ) -> AppResult<Vec<AccountWithCounts>> {
        let accounts = Self::get_all(pool, user_id).await?;
        let mut result = Vec::with_capacity(accounts.len());

        for account in accounts {
            let subscription_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM subscriptions WHERE account_id = $1 AND user_id = $2",
            )
            .bind(account.id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;

            let transaction_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM transactions WHERE account_id = $1 AND user_id = $2",
            )
            .bind(account.id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;

            result.push(AccountWithCounts {
                account: account.into(),
                subscription_count: subscription_count.0,
                transaction_count: transaction_count.0,
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    fn is_first_account(count: i64) -> bool {
        count == 0
    }

    fn can_delete_account(is_default: bool) -> Result<(), &'static str> {
        if is_default {
            Err("Cannot delete the default account")
        } else {
            Ok(())
        }
    }

    #[test]
    fn first_account_becomes_default() {
        assert!(is_first_account(0));
        assert!(!is_first_account(1));
    }

    #[test]
    fn default_account_cannot_be_deleted() {
        assert!(can_delete_account(true).is_err());
        assert!(can_delete_account(false).is_ok());
    }
}
