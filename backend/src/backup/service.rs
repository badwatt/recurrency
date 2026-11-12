use sqlx::PgPool;
use uuid::Uuid;

use crate::accounts::models::Account;
use crate::backup::models::{BACKUP_FORMAT_VERSION, ImportResult, UserBackup};
use crate::categories::models::Category;
use crate::debts::models::{Debt, DebtPayment};
use crate::shared::error::{AppError, AppResult};
use crate::subscriptions::models::Subscription;
use crate::transactions::models::Transaction;

const MAX_BACKUP_ROWS: usize = 100_000;

#[async_trait::async_trait]
pub trait BackupServiceTrait: Send + Sync {
    async fn export(&self, user_id: Uuid) -> AppResult<UserBackup>;
    async fn import(&self, user_id: Uuid, backup: UserBackup) -> AppResult<ImportResult>;
}

pub struct SqlxBackupService {
    pool: PgPool,
}

impl SqlxBackupService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BackupServiceTrait for SqlxBackupService {
    async fn export(&self, user_id: Uuid) -> AppResult<UserBackup> {
        let accounts = sqlx::query_as::<_, Account>(
            "SELECT id, user_id, name, icon, color, is_default, created_at, updated_at FROM accounts WHERE user_id = $1 ORDER BY created_at",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        let categories = sqlx::query_as::<_, Category>(
            "SELECT id, user_id, name, icon, color, is_system, is_default, created_at, updated_at FROM categories WHERE user_id = $1 ORDER BY created_at",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        let subscriptions = sqlx::query_as::<_, Subscription>(
            "SELECT id, user_id, account_id, category_id, name, amount, currency, frequency, schedule_day, schedule_month, recurrence, start_date, end_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at FROM subscriptions WHERE user_id = $1 ORDER BY created_at",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        let transactions = sqlx::query_as::<_, Transaction>(
            "SELECT id, user_id, account_id, category_id, name, amount, currency, date, notes, logo_url, created_at, updated_at FROM transactions WHERE user_id = $1 ORDER BY date",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        let debts = sqlx::query_as::<_, Debt>(
            "SELECT id, user_id, account_id, category_id, name, currency, total_amount, remaining_amount, debt_type, monthly_payment, payment_days, start_date, next_payment_date, subscription_id, is_active, notes, logo_url, created_at, updated_at FROM debts WHERE user_id = $1 ORDER BY created_at",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        let debt_payments = sqlx::query_as::<_, DebtPayment>(
            "SELECT id, debt_id, user_id, amount, paid_on, note, created_at FROM debt_payments WHERE user_id = $1 ORDER BY paid_on",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(UserBackup {
            format_version: BACKUP_FORMAT_VERSION,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            exported_at: chrono::Utc::now(),
            accounts,
            categories,
            subscriptions,
            transactions,
            debts,
            debt_payments,
        })
    }

    async fn import(&self, user_id: Uuid, backup: UserBackup) -> AppResult<ImportResult> {
        if backup.format_version != BACKUP_FORMAT_VERSION {
            return Err(AppError::BadRequest(
                "Unsupported backup format version".to_string(),
            ));
        }
        if backup.accounts.is_empty() || backup.categories.is_empty() {
            return Err(AppError::BadRequest(
                "Backup must contain at least one account and category".to_string(),
            ));
        }
        let total = backup.accounts.len()
            + backup.categories.len()
            + backup.subscriptions.len()
            + backup.transactions.len()
            + backup.debts.len()
            + backup.debt_payments.len();
        if total > MAX_BACKUP_ROWS {
            return Err(AppError::BadRequest("Backup is too large".to_string()));
        }

        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM debt_payments WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM debts WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM transactions WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM subscriptions WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM accounts WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM categories WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        for account in &backup.accounts {
            sqlx::query(
                "INSERT INTO accounts (id, user_id, name, icon, color, is_default, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
            )
            .bind(account.id)
            .bind(user_id)
            .bind(&account.name)
            .bind(&account.icon)
            .bind(&account.color)
            .bind(account.is_default)
            .bind(account.created_at)
            .bind(account.updated_at)
            .execute(&mut *tx)
            .await?;
        }
        for category in &backup.categories {
            sqlx::query(
                "INSERT INTO categories (id, user_id, name, icon, color, is_system, is_default, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
            )
            .bind(category.id)
            .bind(user_id)
            .bind(&category.name)
            .bind(&category.icon)
            .bind(&category.color)
            .bind(category.is_system)
            .bind(category.is_default)
            .bind(category.created_at)
            .bind(category.updated_at)
            .execute(&mut *tx)
            .await?;
        }
        for subscription in &backup.subscriptions {
            sqlx::query(
                "INSERT INTO subscriptions (id, user_id, account_id, category_id, name, amount, currency, frequency, schedule_day, schedule_month, recurrence, start_date, end_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)",
            )
            .bind(subscription.id)
            .bind(user_id)
            .bind(subscription.account_id)
            .bind(subscription.category_id)
            .bind(&subscription.name)
            .bind(subscription.amount)
            .bind(&subscription.currency)
            .bind(&subscription.frequency)
            .bind(subscription.schedule_day)
            .bind(subscription.schedule_month)
            .bind(&subscription.recurrence)
            .bind(subscription.start_date)
            .bind(subscription.end_date)
            .bind(subscription.next_billing_date)
            .bind(subscription.is_active)
            .bind(&subscription.notes)
            .bind(&subscription.logo_url)
            .bind(subscription.created_at)
            .bind(subscription.updated_at)
            .execute(&mut *tx)
            .await?;
        }
        for transaction in &backup.transactions {
            sqlx::query(
                "INSERT INTO transactions (id, user_id, account_id, category_id, name, amount, currency, date, notes, logo_url, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
            )
            .bind(transaction.id)
            .bind(user_id)
            .bind(transaction.account_id)
            .bind(transaction.category_id)
            .bind(&transaction.name)
            .bind(transaction.amount)
            .bind(&transaction.currency)
            .bind(transaction.date)
            .bind(&transaction.notes)
            .bind(&transaction.logo_url)
            .bind(transaction.created_at)
            .bind(transaction.updated_at)
            .execute(&mut *tx)
            .await?;
        }
        for debt in &backup.debts {
            sqlx::query(
                "INSERT INTO debts (id, user_id, account_id, category_id, name, currency, total_amount, remaining_amount, debt_type, monthly_payment, payment_days, start_date, next_payment_date, subscription_id, is_active, notes, logo_url, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)",
            )
            .bind(debt.id)
            .bind(user_id)
            .bind(debt.account_id)
            .bind(debt.category_id)
            .bind(&debt.name)
            .bind(&debt.currency)
            .bind(debt.total_amount)
            .bind(debt.remaining_amount)
            .bind(&debt.debt_type)
            .bind(debt.monthly_payment)
            .bind(&debt.payment_days)
            .bind(debt.start_date)
            .bind(debt.next_payment_date)
            .bind(debt.subscription_id)
            .bind(debt.is_active)
            .bind(&debt.notes)
            .bind(&debt.logo_url)
            .bind(debt.created_at)
            .bind(debt.updated_at)
            .execute(&mut *tx)
            .await?;
        }
        for payment in &backup.debt_payments {
            sqlx::query(
                "INSERT INTO debt_payments (id, debt_id, user_id, amount, paid_on, note, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7)",
            )
            .bind(payment.id)
            .bind(payment.debt_id)
            .bind(user_id)
            .bind(payment.amount)
            .bind(payment.paid_on)
            .bind(&payment.note)
            .bind(payment.created_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(ImportResult {
            accounts: backup.accounts.len(),
            categories: backup.categories.len(),
            subscriptions: backup.subscriptions.len(),
            transactions: backup.transactions.len(),
            debts: backup.debts.len(),
            debt_payments: backup.debt_payments.len(),
        })
    }
}
