use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::accounts::service::AccountService;
use crate::categories::service::CategoryService;
use crate::debts::models::{
    CreateDebtRequest, Debt, DebtPayment, DebtResponse, RecordDebtPaymentRequest,
    UpdateDebtRequest, validate_debt_type,
};
use crate::shared::error::{AppError, AppResult};
use crate::subscriptions::models::{CreateSubscriptionRequest, Recurrence};
use crate::subscriptions::service::SubscriptionService;
use crate::transactions::models::CreateTransactionRequest;
use crate::transactions::service::TransactionService;

#[async_trait::async_trait]
pub trait DebtServiceTrait: Send + Sync {
    async fn create(&self, user_id: Uuid, req: CreateDebtRequest) -> AppResult<DebtResponse>;
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<DebtResponse>>;
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<DebtResponse>;
    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateDebtRequest,
    ) -> AppResult<DebtResponse>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn record_payment(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: RecordDebtPaymentRequest,
    ) -> AppResult<DebtResponse>;
    async fn payments(&self, user_id: Uuid, id: Uuid) -> AppResult<Vec<DebtPayment>>;
}

pub struct SqlxDebtService {
    pool: PgPool,
}

impl SqlxDebtService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DebtServiceTrait for SqlxDebtService {
    async fn create(&self, user_id: Uuid, req: CreateDebtRequest) -> AppResult<DebtResponse> {
        DebtService::create(&self.pool, user_id, req).await
    }
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<DebtResponse>> {
        DebtService::get_all(&self.pool, user_id).await
    }
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<DebtResponse> {
        DebtService::get_by_id(&self.pool, user_id, id).await
    }
    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateDebtRequest,
    ) -> AppResult<DebtResponse> {
        DebtService::update(&self.pool, user_id, id, req).await
    }
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        DebtService::delete(&self.pool, user_id, id).await
    }
    async fn record_payment(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: RecordDebtPaymentRequest,
    ) -> AppResult<DebtResponse> {
        DebtService::record_payment(&self.pool, user_id, id, req).await
    }
    async fn payments(&self, user_id: Uuid, id: Uuid) -> AppResult<Vec<DebtPayment>> {
        DebtService::payments(&self.pool, user_id, id).await
    }
}

pub struct DebtService;

impl DebtService {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        req: CreateDebtRequest,
    ) -> AppResult<DebtResponse> {
        let debt_type = req.debt_type.trim().to_lowercase();
        if !validate_debt_type(&debt_type) {
            return Err(AppError::BadRequest(
                "Debt type must be manual or automatic".to_string(),
            ));
        }
        if req.total_amount <= Decimal::ZERO {
            return Err(AppError::BadRequest(
                "Debt amount must be greater than zero".to_string(),
            ));
        }
        if req.name.trim().is_empty() {
            return Err(AppError::BadRequest("Debt name is required".to_string()));
        }
        let account_id = if req.account_id.is_nil() {
            AccountService::get_default(pool, user_id).await?.id
        } else {
            AccountService::get_by_id(pool, user_id, req.account_id).await?.id
        };
        let category_id = match req.category_id {
            Some(id) => CategoryService::get_by_id(pool, user_id, id).await?.id,
            None => CategoryService::get_default(pool, user_id).await?.id,
        };

        let start_date = req.start_date.unwrap_or_else(|| Utc::now().date_naive());
        let (monthly_payment, next_payment_date, subscription_id) = if debt_type == "automatic" {
            let monthly = req.monthly_payment.ok_or_else(|| {
                AppError::BadRequest("Automatic debts need a monthly payment".to_string())
            })?;
            if monthly <= Decimal::ZERO {
                return Err(AppError::BadRequest(
                    "Monthly payment must be greater than zero".to_string(),
                ));
            }
            let recurrence = Recurrence {
                interval: 1,
                unit: "month".into(),
                days: vec![start_date.day() as u8],
                ..Default::default()
            };
            let subscription = SubscriptionService::create(
                pool,
                user_id,
                CreateSubscriptionRequest {
                    name: req.name.clone(),
                    amount: monthly,
                    currency: req.currency.clone(),
                    frequency: "monthly".into(),
                    schedule_day: Some(start_date.day() as i16),
                    schedule_month: None,
                    recurrence: Some(recurrence),
                    is_active: Some(true),
                    start_date,
                    end_date: None,
                    account_id,
                    category_id: Some(category_id),
                    notes: Some("Automatic debt payment".to_string()),
                    logo_url: req.logo_url.clone(),
                },
            )
            .await?;
            (Some(monthly), Some(start_date), Some(subscription.id))
        } else {
            (None, None, None)
        };

        let debt = sqlx::query_as::<_, Debt>(
            r#"
            INSERT INTO debts (id, user_id, account_id, category_id, name, currency,
                total_amount, remaining_amount, debt_type, monthly_payment,
                next_payment_date, subscription_id, is_active, notes, logo_url, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$7,$8,$9,$10,$11,true,$12,$13,$14,$14)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(account_id)
        .bind(category_id)
        .bind(req.name.trim())
        .bind(req.currency.to_uppercase())
        .bind(req.total_amount)
        .bind(&debt_type)
        .bind(monthly_payment)
        .bind(next_payment_date)
        .bind(subscription_id)
        .bind(req.notes)
        .bind(req.logo_url)
        .bind(Utc::now())
        .fetch_one(pool)
        .await?;
        Ok(debt.into())
    }

    pub async fn get_all(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<DebtResponse>> {
        Self::process_due(pool, user_id).await?;
        let debts = sqlx::query_as::<_, Debt>(
            "SELECT * FROM debts WHERE user_id = $1 ORDER BY is_active DESC, created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(debts.into_iter().map(Into::into).collect())
    }

    pub async fn get_by_id(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<DebtResponse> {
        Self::process_due(pool, user_id).await?;
        let debt = sqlx::query_as::<_, Debt>("SELECT * FROM debts WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Debt not found".to_string()))?;
        Ok(debt.into())
    }

    pub async fn update(
        pool: &PgPool,
        user_id: Uuid,
        id: Uuid,
        req: UpdateDebtRequest,
    ) -> AppResult<DebtResponse> {
        let existing = Self::get_by_id(pool, user_id, id).await?;
        let name = req.name.unwrap_or(existing.name);
        let notes = if req.clear_notes {
            None
        } else {
            match req.notes {
                Some(value) if value.trim().is_empty() => None,
                Some(value) => Some(value),
                None => existing.notes,
            }
        };
        let logo_url = req.logo_url.or(existing.logo_url);
        let debt = sqlx::query_as::<_, Debt>(
            "UPDATE debts SET name=$1, notes=$2, logo_url=$3, updated_at=NOW() WHERE id=$4 AND user_id=$5 RETURNING *",
        )
        .bind(name)
        .bind(notes)
        .bind(logo_url)
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(debt.into())
    }

    pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<()> {
        let debt = sqlx::query_as::<_, Debt>("SELECT * FROM debts WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Debt not found".to_string()))?;
        if let Some(subscription_id) = debt.subscription_id {
            let _ = SubscriptionService::delete(pool, user_id, subscription_id).await;
        }
        sqlx::query("DELETE FROM debts WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn record_payment(
        pool: &PgPool,
        user_id: Uuid,
        id: Uuid,
        req: RecordDebtPaymentRequest,
    ) -> AppResult<DebtResponse> {
        if req.amount <= Decimal::ZERO {
            return Err(AppError::BadRequest(
                "Payment must be greater than zero".to_string(),
            ));
        }
        let debt = sqlx::query_as::<_, Debt>("SELECT * FROM debts WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Debt not found".to_string()))?;
        if debt.remaining_amount <= Decimal::ZERO {
            return Err(AppError::BadRequest("Debt is already paid".to_string()));
        }
        if req.amount > debt.remaining_amount {
            return Err(AppError::BadRequest(
                "Payment exceeds the remaining debt".to_string(),
            ));
        }
        let remaining = debt.remaining_amount - req.amount;
        let mut tx = pool.begin().await?;
        sqlx::query(
            "INSERT INTO debt_payments (id, debt_id, user_id, amount, paid_on, note) VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(user_id)
        .bind(req.amount)
        .bind(req.paid_on)
        .bind(&req.note)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE debts SET remaining_amount=$1, is_active=$2, updated_at=NOW() WHERE id=$3 AND user_id=$4",
        )
        .bind(remaining)
        .bind(remaining > Decimal::ZERO)
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        // Manual payments become normal transactions so reports see the expense.
        // Automatic debts are already represented by their linked subscription.
        if debt.debt_type == "manual" {
            let _ = TransactionService::create(
                pool,
                user_id,
                CreateTransactionRequest {
                    name: format!("Debt payment: {}", debt.name),
                    amount: req.amount,
                    currency: debt.currency.clone(),
                    date: req.paid_on,
                    account_id: debt.account_id,
                    category_id: Some(debt.category_id),
                    notes: req.note.clone(),
                    logo_url: None,
                },
            )
            .await;
        }

        Self::get_by_id(pool, user_id, id).await
    }

    pub async fn payments(
        pool: &PgPool,
        user_id: Uuid,
        id: Uuid,
    ) -> AppResult<Vec<DebtPayment>> {
        let exists: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM debts WHERE id = $1 AND user_id = $2")
                .bind(id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?;
        if exists.is_none() {
            return Err(AppError::NotFound("Debt not found".to_string()));
        }
        Ok(sqlx::query_as::<_, DebtPayment>(
            "SELECT * FROM debt_payments WHERE debt_id = $1 AND user_id = $2 ORDER BY paid_on DESC, created_at DESC",
        )
        .bind(id)
        .bind(user_id)
        .fetch_all(pool)
        .await?)
    }

    async fn process_due(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
        let today = Utc::now().date_naive();
        let due = sqlx::query_as::<_, Debt>(
            "SELECT * FROM debts WHERE user_id = $1 AND debt_type = 'automatic' AND is_active = true AND remaining_amount > 0 AND next_payment_date IS NOT NULL AND next_payment_date <= $2",
        )
        .bind(user_id)
        .bind(today)
        .fetch_all(pool)
        .await?;

        for debt in due {
            let Some(monthly) = debt.monthly_payment else {
                continue;
            };
            if monthly <= Decimal::ZERO {
                continue;
            }
            let mut remaining = debt.remaining_amount;
            let mut next = debt
                .next_payment_date
                .ok_or_else(|| AppError::Internal("Missing next payment date".into()))?;
            let mut last_paid = next;
            let mut tx = pool.begin().await?;
            while next <= today && remaining > Decimal::ZERO {
                let amount = monthly.min(remaining);
                sqlx::query(
                    "INSERT INTO debt_payments (id, debt_id, user_id, amount, paid_on, note) VALUES ($1,$2,$3,$4,$5,$6)",
                )
                .bind(Uuid::new_v4())
                .bind(debt.id)
                .bind(user_id)
                .bind(amount)
                .bind(next)
                .bind("Automatic payment")
                .execute(&mut *tx)
                .await?;
                remaining -= amount;
                last_paid = next;
                next = add_month(next);
            }
            sqlx::query(
                "UPDATE debts SET remaining_amount=$1, is_active=$2, next_payment_date=$3, updated_at=NOW() WHERE id=$4 AND user_id=$5",
            )
            .bind(remaining)
            .bind(remaining > Decimal::ZERO)
            .bind((remaining > Decimal::ZERO).then_some(next))
            .bind(debt.id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
            if remaining <= Decimal::ZERO
                && let Some(subscription_id) = debt.subscription_id
            {
                sqlx::query(
                    "UPDATE subscriptions SET is_active=false, end_date=$1, next_billing_date=NULL, updated_at=NOW() WHERE id=$2 AND user_id=$3",
                )
                .bind(last_paid)
                .bind(subscription_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
            }
            tx.commit().await?;
        }
        Ok(())
    }
}

fn add_month(date: NaiveDate) -> NaiveDate {
    let total = date.year() * 12 + date.month() as i32;
    let year = total.div_euclid(12);
    let month = (total.rem_euclid(12) + 1) as u32;
    let day = date.day().min(crate::shared::date_utils::days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_addition_clamps_short_months() {
        let jan = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        assert_eq!(add_month(jan), NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }
}
