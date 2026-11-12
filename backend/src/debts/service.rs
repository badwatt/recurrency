use chrono::{Datelike, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::accounts::service::AccountService;
use crate::categories::service::CategoryService;
use crate::debts::models::{
    CreateDebtRequest, DEBT_TYPE_AUTOMATIC, DEBT_TYPE_MANUAL, Debt, DebtPayment, DebtResponse,
    RecordDebtPaymentRequest, UpdateDebtRequest, validate_debt_type,
};
use crate::shared::error::{AppError, AppResult};
use crate::subscriptions::models::{CreateSubscriptionRequest, Recurrence, UpdateSubscriptionRequest};
use crate::subscriptions::recurrence::{next_occurrence_on_or_after, validate_recurrence};
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
        let mut payment_days = req.payment_days.clone();
        if debt_type == DEBT_TYPE_AUTOMATIC {
            if payment_days.is_empty() {
                payment_days.push(start_date.day() as u8);
            }
            payment_days.sort_unstable();
            payment_days.dedup();
            if payment_days.iter().any(|day| *day > 31) {
                return Err(AppError::BadRequest(
                    "Payment days must be 0-31 (0 = last day)".to_string(),
                ));
            }
        }
        let (monthly_payment, next_payment_date, subscription_id) = if debt_type == DEBT_TYPE_AUTOMATIC {
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
                days: payment_days.clone(),
                ..Default::default()
            };
            validate_recurrence(&recurrence)?;
            let subscription = SubscriptionService::create(
                pool,
                user_id,
                CreateSubscriptionRequest {
                    name: req.name.clone(),
                    amount: monthly,
                    currency: req.currency.clone(),
                    frequency: "monthly".into(),
                    schedule_day: Some(if payment_days.first() == Some(&0) {
                        31
                    } else {
                        i16::from(
                            payment_days
                                .first()
                                .copied()
                                .unwrap_or(start_date.day() as u8),
                        )
                    }),
                    schedule_month: None,
                    recurrence: Some(recurrence.clone()),
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
            let next = next_occurrence_on_or_after(start_date, &recurrence, start_date, None)
                .unwrap_or(start_date);
            (Some(monthly), Some(next), Some(subscription.id))
        } else {
            (None, None, None)
        };

        let debt = sqlx::query_as::<_, Debt>(
            r#"
            INSERT INTO debts (id, user_id, account_id, category_id, name, currency,
                total_amount, remaining_amount, debt_type, monthly_payment,
                payment_days, start_date, next_payment_date, subscription_id, is_active, notes, logo_url, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$7,$8,$9,$10,$11,$12,$13,true,$14,$15,$16,$16)
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
        .bind(sqlx::types::Json(&payment_days))
        .bind(start_date)
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
        let debt_type = req
            .debt_type
            .unwrap_or_else(|| existing.debt_type.clone())
            .trim()
            .to_lowercase();
        if !validate_debt_type(&debt_type) {
            return Err(AppError::BadRequest(
                "Debt type must be manual or automatic".to_string(),
            ));
        }
        let total_amount = req.total_amount.unwrap_or(existing.total_amount);
        if total_amount <= Decimal::ZERO {
            return Err(AppError::BadRequest(
                "Debt amount must be greater than zero".to_string(),
            ));
        }
        let paid = existing.total_amount - existing.remaining_amount;
        let remaining_amount = if total_amount == existing.total_amount {
            existing.remaining_amount
        } else {
            (total_amount - paid).max(Decimal::ZERO)
        };
        let currency = req
            .currency
            .unwrap_or_else(|| existing.currency.clone())
            .to_uppercase();
        let account_id = match req.account_id {
            Some(account_id) if !account_id.is_nil() => {
                AccountService::get_by_id(pool, user_id, account_id).await?.id
            }
            _ => existing.account_id,
        };
        let category_id = match req.category_id {
            Some(category_id) => {
                CategoryService::get_by_id(pool, user_id, category_id).await?.id
            }
            None => existing.category_id,
        };
        let start_date = req.start_date.unwrap_or(existing.start_date);
        let mut payment_days = req
            .payment_days
            .unwrap_or_else(|| existing.payment_days.clone());
        if debt_type == DEBT_TYPE_AUTOMATIC {
            if payment_days.is_empty() {
                payment_days.push(start_date.day() as u8);
            }
            payment_days.sort_unstable();
            payment_days.dedup();
            if payment_days.iter().any(|day| *day > 31) {
                return Err(AppError::BadRequest(
                    "Payment days must be 0-31 (0 = last day)".to_string(),
                ));
            }
        } else {
            payment_days.clear();
        }
        let recurrence = Recurrence {
            interval: 1,
            unit: "month".into(),
            days: payment_days.clone(),
            ..Default::default()
        };
        if debt_type == DEBT_TYPE_AUTOMATIC {
            validate_recurrence(&recurrence)?;
        }

        let mut monthly_payment = None;
        let mut next_payment_date = None;
        let mut subscription_id = existing.subscription_id;
        if debt_type == DEBT_TYPE_AUTOMATIC {
            let monthly = req
                .monthly_payment
                .or(existing.monthly_payment)
                .ok_or_else(|| {
                    AppError::BadRequest("Automatic debts need a monthly payment".to_string())
                })?;
            if monthly <= Decimal::ZERO {
                return Err(AppError::BadRequest(
                    "Monthly payment must be greater than zero".to_string(),
                ));
            }
            monthly_payment = Some(monthly);
            let schedule_changed = payment_days != existing.payment_days
                || start_date != existing.start_date;
            let schedule_day = if payment_days.first() == Some(&0) {
                31
            } else {
                i16::from(
                    payment_days
                        .first()
                        .copied()
                        .unwrap_or(start_date.day() as u8),
                )
            };
            if let Some(existing_subscription) = existing.subscription_id {
                SubscriptionService::update(
                    pool,
                    user_id,
                    existing_subscription,
                    UpdateSubscriptionRequest {
                        name: req.name.clone(),
                        amount: Some(monthly),
                        currency: Some(currency.clone()),
                        frequency: Some("monthly".to_string()),
                        schedule_day: Some(schedule_day),
                        recurrence: Some(recurrence.clone()),
                        start_date: Some(start_date),
                        account_id: Some(account_id),
                        category_id: Some(category_id),
                        ..Default::default()
                    },
                )
                .await?;
                if schedule_changed {
                    next_payment_date = next_occurrence_on_or_after(
                        start_date,
                        &recurrence,
                        Utc::now().date_naive(),
                        None,
                    );
                } else {
                    next_payment_date = existing.next_payment_date;
                }
            } else {
                let subscription = SubscriptionService::create(
                    pool,
                    user_id,
                    CreateSubscriptionRequest {
                        name: req.name.clone().unwrap_or_else(|| existing.name.clone()),
                        amount: monthly,
                        currency: currency.clone(),
                        frequency: "monthly".into(),
                        schedule_day: Some(schedule_day),
                        schedule_month: None,
                        recurrence: Some(recurrence.clone()),
                        is_active: Some(true),
                        start_date,
                        end_date: None,
                        account_id,
                        category_id: Some(category_id),
                        notes: Some("Automatic debt payment".to_string()),
                        logo_url: req.logo_url.clone().or_else(|| existing.logo_url.clone()),
                    },
                )
                .await?;
                subscription_id = Some(subscription.id);
                next_payment_date =
                    next_occurrence_on_or_after(start_date, &recurrence, start_date, None);
            }
            if remaining_amount <= Decimal::ZERO
                && let Some(subscription_id) = subscription_id
            {
                let _ = SubscriptionService::update(
                    pool,
                    user_id,
                    subscription_id,
                    UpdateSubscriptionRequest {
                        is_active: Some(false),
                        end_date: Some(existing.start_date),
                        ..Default::default()
                    },
                )
                .await;
            }
        } else if let Some(linked_subscription) = subscription_id {
            let _ = SubscriptionService::delete(pool, user_id, linked_subscription).await;
            subscription_id = None;
        }

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
        let is_active = remaining_amount > Decimal::ZERO;
        let debt = sqlx::query_as::<_, Debt>(
            r#"
            UPDATE debts SET name=$1, total_amount=$2, remaining_amount=$3, currency=$4,
                account_id=$5, category_id=$6, debt_type=$7, monthly_payment=$8,
                payment_days=$9, start_date=$10, next_payment_date=$11,
                subscription_id=$12, is_active=$13, notes=$14, logo_url=$15, updated_at=NOW()
            WHERE id=$16 AND user_id=$17
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(total_amount)
        .bind(remaining_amount)
        .bind(currency)
        .bind(account_id)
        .bind(category_id)
        .bind(&debt_type)
        .bind(monthly_payment)
        .bind(sqlx::types::Json(&payment_days))
        .bind(start_date)
        .bind(next_payment_date)
        .bind(subscription_id)
        .bind(is_active)
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
        if debt.debt_type == DEBT_TYPE_MANUAL {
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
            let mut days = debt.payment_days.0.clone();
            if days.is_empty() {
                days.push(debt.start_date.day() as u8);
            }
            let recurrence = Recurrence {
                interval: 1,
                unit: "month".into(),
                days,
                ..Default::default()
            };
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
                let Some(following) = next_occurrence_on_or_after(
                    debt.start_date,
                    &recurrence,
                    next + chrono::Duration::days(1),
                    None,
                ) else {
                    break;
                };
                next = following;
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


