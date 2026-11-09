use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::categories::service::CategoryService;
use crate::shared::date_utils::days_in_month;
use crate::shared::error::{AppError, AppResult};
use crate::subscriptions::models::{
    CreateSubscriptionRequest, Subscription, UpdateSubscriptionRequest,
};

pub struct SubscriptionService;

/// Subscription port, injected into `AppState` as `Arc<dyn SubscriptionServiceTrait>`.
#[async_trait::async_trait]
pub trait SubscriptionServiceTrait: Send + Sync {
    async fn create(&self, user_id: Uuid, req: CreateSubscriptionRequest) -> AppResult<Subscription>;
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Subscription>>;
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Subscription>;
    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateSubscriptionRequest,
    ) -> AppResult<Subscription>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
    async fn get_upcoming(&self, user_id: Uuid, days: i64) -> AppResult<Vec<Subscription>>;
    async fn get_calendar(&self, user_id: Uuid, year: i32, month: u32)
    -> AppResult<Vec<CalendarItem>>;
}

/// SQLx adapter delegating to the existing, tested [`SubscriptionService`] functions.
pub struct SqlxSubscriptionService {
    pool: PgPool,
}

impl SqlxSubscriptionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionServiceTrait for SqlxSubscriptionService {
    async fn create(&self, user_id: Uuid, req: CreateSubscriptionRequest) -> AppResult<Subscription> {
        SubscriptionService::create(&self.pool, user_id, req).await
    }

    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<Subscription>> {
        SubscriptionService::get_all(&self.pool, user_id).await
    }

    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<Subscription> {
        SubscriptionService::get_by_id(&self.pool, user_id, id).await
    }

    async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateSubscriptionRequest,
    ) -> AppResult<Subscription> {
        SubscriptionService::update(&self.pool, user_id, id, req).await
    }

    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()> {
        SubscriptionService::delete(&self.pool, user_id, id).await
    }

    async fn get_upcoming(&self, user_id: Uuid, days: i64) -> AppResult<Vec<Subscription>> {
        SubscriptionService::get_upcoming(&self.pool, user_id, days).await
    }

    async fn get_calendar(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<Vec<CalendarItem>> {
        SubscriptionService::get_calendar(&self.pool, user_id, year, month).await
    }
}

impl SubscriptionService {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        req: CreateSubscriptionRequest,
    ) -> AppResult<Subscription> {
        let next_billing_date = calculate_next_billing_date(req.start_date, &req.frequency);

        // Accounts are mandatory; fall back to the user's default for old clients.
        let account_id = if req.account_id.is_nil() {
            crate::accounts::service::AccountService::get_default(pool, user_id)
                .await?
                .id
        } else {
            req.account_id
        };

        // Auto-assign default category if none provided
        let category_id = match req.category_id {
            Some(id) => id,
            None => CategoryService::get_default(pool, user_id).await?.id,
        };

        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            INSERT INTO subscriptions (id, user_id, account_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, $11, $12, $13, $13)
            RETURNING id, user_id, account_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(account_id)
        .bind(category_id)
        .bind(&req.name)
        .bind(req.amount)
        .bind(&req.currency)
        .bind(&req.frequency)
        .bind(req.start_date)
        .bind(next_billing_date)
        .bind(&req.notes)
        .bind(&req.logo_url)
        .bind(Utc::now())
        .fetch_one(pool)
        .await?;

        Ok(subscription)
    }

    pub async fn get_all(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Subscription>> {
        let subscriptions = sqlx::query_as::<_, Subscription>(
            "SELECT id, user_id, account_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at FROM subscriptions WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(subscriptions)
    }

    pub async fn get_by_id(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Subscription> {
        let subscription = sqlx::query_as::<_, Subscription>(
            "SELECT id, user_id, account_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at FROM subscriptions WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Subscription not found".to_string()))?;

        Ok(subscription)
    }

    pub async fn update(
        pool: &PgPool,
        user_id: Uuid,
        id: Uuid,
        req: UpdateSubscriptionRequest,
    ) -> AppResult<Subscription> {
        // First verify the subscription exists and belongs to user
        let existing = Self::get_by_id(pool, user_id, id).await?;

        let name = req.name.unwrap_or(existing.name);
        let amount = req.amount.unwrap_or(existing.amount);
        let currency = req.currency.unwrap_or(existing.currency);
        let frequency_changed = req.frequency.is_some()
            && req.frequency.as_ref() != Some(&existing.frequency);
        let frequency = req.frequency.unwrap_or(existing.frequency.clone());
        let start_date = req.start_date.unwrap_or(existing.start_date);
        let next_billing_date = if let Some(nbd) = req.next_billing_date {
            nbd
        } else if frequency_changed {
            // Frequency changed: recalculate from existing next_billing_date (or start_date)
            calculate_next_billing_date(
                existing.next_billing_date.unwrap_or(start_date),
                &frequency,
            )
        } else {
            existing.next_billing_date.unwrap_or_else(|| {
                calculate_next_billing_date(start_date, &frequency)
            })
        };
        let account_id = req.account_id.unwrap_or(existing.account_id);
        let category_id = req.category_id.unwrap_or(existing.category_id);
        let is_active = req.is_active.unwrap_or(existing.is_active);
        let notes = req.notes.or(existing.notes);
        let logo_url = req.logo_url.or(existing.logo_url);

        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            UPDATE subscriptions 
            SET name = $1, amount = $2, currency = $3, frequency = $4, start_date = $5, 
                next_billing_date = $6, account_id = $7, category_id = $8, is_active = $9, notes = $10, logo_url = $11, updated_at = $12
            WHERE id = $13 AND user_id = $14
            RETURNING id, user_id, account_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(amount)
        .bind(&currency)
        .bind(&frequency)
        .bind(start_date)
        .bind(next_billing_date)
        .bind(account_id)
        .bind(category_id)
        .bind(is_active)
        .bind(&notes)
        .bind(&logo_url)
        .bind(Utc::now())
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(subscription)
    }

    pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM subscriptions WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Subscription not found".to_string()));
        }

        Ok(())
    }

    pub async fn get_upcoming(
        pool: &PgPool,
        user_id: Uuid,
        days: i64,
    ) -> AppResult<Vec<Subscription>> {
        let from_date = Utc::now().date_naive();
        let to_date = from_date + chrono::Duration::days(days);

        let subscriptions = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT id, user_id, account_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at 
            FROM subscriptions 
            WHERE user_id = $1 AND is_active = true AND next_billing_date >= $2 AND next_billing_date <= $3
            ORDER BY next_billing_date ASC
            "#,
        )
        .bind(user_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(pool)
        .await?;

        Ok(subscriptions)
    }

    pub async fn get_calendar(
        pool: &PgPool,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<Vec<CalendarItem>> {
        // Get all active subscriptions
        let subscriptions = sqlx::query_as::<_, Subscription>(
            "SELECT id, user_id, account_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at FROM subscriptions WHERE user_id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        // Calculate billing dates for the given month
        let month_start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let month_end = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
        };

        let mut calendar_items: Vec<CalendarItem> = Vec::new();

        for sub in subscriptions {
            let mut billing_date = sub.next_billing_date.unwrap_or(sub.start_date);

            while billing_date < month_start {
                billing_date = advance_date(billing_date, &sub.frequency);
            }

            while billing_date <= month_end {
                calendar_items.push(CalendarItem {
                    subscription_id: sub.id,
                    name: sub.name.clone(),
                    amount: sub.amount,
                    currency: sub.currency.clone(),
                    frequency: sub.frequency.clone(),
                    billing_date,
                    is_active: sub.is_active,
                    logo_url: sub.logo_url.clone(),
                });
                billing_date = advance_date(billing_date, &sub.frequency);
            }
        }

        sort_calendar_items(&mut calendar_items);

        Ok(calendar_items)
    }
}

#[derive(Debug, Serialize)]
pub struct CalendarItem {
    pub subscription_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub billing_date: NaiveDate,
    pub is_active: bool,
    pub logo_url: Option<String>,
}

pub(crate) fn sort_calendar_items(items: &mut [CalendarItem]) {
    items.sort_by_key(|a| a.billing_date);
}

/// Generate billing dates for a given subscription within a month.
#[allow(dead_code)]
pub(crate) fn generate_billing_dates(
    next_billing: NaiveDate,
    frequency: &str,
    month_start: NaiveDate,
    month_end: NaiveDate,
) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let mut billing_date = next_billing;

    while billing_date < month_start {
        billing_date = advance_date(billing_date, frequency);
    }

    while billing_date <= month_end {
        dates.push(billing_date);
        billing_date = advance_date(billing_date, frequency);
    }

    dates
}

pub(crate) fn advance_date(date: NaiveDate, frequency: &str) -> NaiveDate {
    match frequency {
        "weekly" => date + chrono::Duration::weeks(1),
        "monthly" => add_months(date, 1),
        "quarterly" => add_months(date, 3),
        "yearly" => add_months(date, 12),
        _ => date + chrono::Duration::days(30),
    }
}

pub(crate) fn calculate_next_billing_date(start_date: NaiveDate, frequency: &str) -> NaiveDate {
    let today = Utc::now().date_naive();
    let mut next_date = start_date;
    let mut iterations = 0;

    while next_date <= today {
        next_date = advance_date(next_date, frequency);
        iterations += 1;
        if iterations > 1000 {
            tracing::warn!(
                "calculate_next_billing_date: loop guard triggered for start_date={} frequency={}",
                start_date, frequency
            );
            break;
        }
    }

    next_date
}

pub(crate) fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let year = date.year();
    let month = date.month() as i32;
    let day = date.day();

    let new_month = ((month - 1 + months) % 12) + 1;
    let year_offset = (month - 1 + months) / 12;
    let new_year = year + year_offset;

    // Handle end-of-month edge cases
    let max_day = days_in_month(new_year, new_month as u32);
    let new_day = day.min(max_day);

    NaiveDate::from_ymd_opt(new_year, new_month as u32, new_day)
        .unwrap_or(date + chrono::Duration::days(30))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    // ── advance_date ──

    #[test]
    fn advance_weekly() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let next = advance_date(d, "weekly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2024, 1, 22).unwrap());
    }

    #[test]
    fn advance_monthly_jan_to_feb() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let next = advance_date(d, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2024, 2, 15).unwrap());
    }

    #[test]
    fn advance_monthly_dec_wraps_year() {
        let d = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
        let next = advance_date(d, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    }

    #[test]
    fn advance_monthly_jan_31_to_feb_28_non_leap() {
        let d = NaiveDate::from_ymd_opt(2023, 1, 31).unwrap();
        let next = advance_date(d, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());
    }

    #[test]
    fn advance_monthly_jan_31_to_feb_29_leap() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let next = advance_date(d, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }

    #[test]
    fn advance_quarterly() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let next = advance_date(d, "quarterly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2024, 4, 15).unwrap());
    }

    #[test]
    fn advance_quarterly_nov_crosses_year() {
        let d = NaiveDate::from_ymd_opt(2024, 11, 15).unwrap();
        let next = advance_date(d, "quarterly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2025, 2, 15).unwrap());
    }

    #[test]
    fn advance_yearly() {
        let d = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let next = advance_date(d, "yearly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
    }

    #[test]
    fn advance_yearly_feb_29_to_non_leap() {
        let d = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let next = advance_date(d, "yearly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2025, 2, 28).unwrap());
    }

    #[test]
    fn advance_unknown_frequency_fallback() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let next = advance_date(d, "biweekly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2024, 2, 14).unwrap()); // +30 days
    }

    // ── add_months ──

    #[test]
    fn add_months_simple() {
        let d = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let result = add_months(d, 2);
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 5, 15).unwrap());
    }

    #[test]
    fn add_months_cross_year() {
        let d = NaiveDate::from_ymd_opt(2024, 11, 15).unwrap();
        let result = add_months(d, 3);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 2, 15).unwrap());
    }

    #[test]
    fn add_months_12_is_one_year() {
        let d = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = add_months(d, 12);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
    }

    #[test]
    fn add_months_day_clamping_jan_31_to_feb() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let result = add_months(d, 1);
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()); // leap year
    }

    #[test]
    fn add_months_day_clamping_may_31_to_june() {
        let d = NaiveDate::from_ymd_opt(2024, 5, 31).unwrap();
        let result = add_months(d, 1);
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 6, 30).unwrap());
    }

    #[test]
    fn add_months_day_clamping_aug_31_to_sep() {
        let d = NaiveDate::from_ymd_opt(2024, 8, 31).unwrap();
        let result = add_months(d, 1);
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 9, 30).unwrap());
    }

    #[test]
    fn add_months_day_clamping_jan_31_to_april() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let result = add_months(d, 3);
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 4, 30).unwrap());
    }

    #[test]
    fn add_months_zero_is_noop() {
        let d = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = add_months(d, 0);
        assert_eq!(result, d);
    }

    #[test]
    fn add_months_preserves_day_within_bounds() {
        let d = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        let result = add_months(d, 6);
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 8, 15).unwrap());
    }

    // ── calculate_next_billing_date ──

    #[test]
    fn next_billing_date_future_start_date() {
        // Start date in the future should return itself
        let future = NaiveDate::from_ymd_opt(2099, 6, 15).unwrap();
        let result = calculate_next_billing_date(future, "monthly");
        assert_eq!(result, future);
    }

    #[test]
    fn next_billing_date_past_weekly_advances_until_future() {
        // Start date far in past, weekly should advance
        let past = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let result = calculate_next_billing_date(past, "weekly");
        assert!(result > chrono::Utc::now().date_naive());
    }

    #[test]
    fn next_billing_date_past_monthly_advances_until_future() {
        let past = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let result = calculate_next_billing_date(past, "monthly");
        assert!(result > chrono::Utc::now().date_naive());
    }

    #[test]
    fn next_billing_date_past_yearly_advances_until_future() {
        let past = NaiveDate::from_ymd_opt(2010, 6, 15).unwrap();
        let result = calculate_next_billing_date(past, "yearly");
        assert!(result >= chrono::Utc::now().date_naive());
    }

    #[test]
    fn next_billing_date_today_returns_advanced() {
        // If start_date is today, we advance at least once (since while is <=)
        let today = chrono::Utc::now().date_naive();
        let result = calculate_next_billing_date(today, "monthly");
        assert!(result > today);
    }

    // ── CalendarItem logic (integration-style, no DB) ──

    #[test]
    fn calendar_billing_loop_within_month() {
        // Simulate weekly frequency billing dates within a specific month
        // Use advance_date directly to avoid calculate_next_billing_date going to future
        let _month_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        // Start from the first billing in January
        let mut d = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap();

        let mut dates = vec![];
        while d <= month_end {
            dates.push(d);
            d = advance_date(d, "weekly");
        }

        // Weekly in January 2024 should have at least 3 billing dates
        assert!(
            dates.len() >= 3,
            "Expected at least 3 weekly dates, got {}",
            dates.len()
        );
        for &dt in &dates {
            assert!(dt.year() == 2024 && dt.month() == 1);
        }
    }

    #[test]
    fn calendar_monthly_single_date_per_month() {
        let month_start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let feb_15 = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        let dates = generate_billing_dates(feb_15, "monthly", month_start, month_end);
        // Since Feb 15 is before month_start (Mar 1), advance to Mar 15
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    // ── generate_billing_dates ──

    #[test]
    fn generate_billing_dates_weekly_in_january() {
        let month_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(),
            "weekly",
            month_start,
            month_end,
        );
        assert!(dates.len() >= 3);
        for d in &dates {
            assert!(*d >= month_start && *d <= month_end);
        }
    }

    #[test]
    fn generate_billing_dates_monthly_in_june() {
        let month_start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            "monthly",
            month_start,
            month_end,
        );
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    }

    #[test]
    fn generate_billing_dates_quarterly_no_occurrence() {
        let month_start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        // Billing on Jan 15, quarterly => next is April 15
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "quarterly",
            month_start,
            month_end,
        );
        assert!(dates.is_empty(), "No quarterly billing in February");
    }

    #[test]
    fn generate_billing_dates_yearly_once_per_year() {
        let month_start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            "yearly",
            month_start,
            month_end,
        );
        assert_eq!(dates.len(), 1);
    }

    #[test]
    fn generate_billing_dates_before_month_start() {
        // next_billing is before month_start, should advance
        let month_start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "monthly",
            month_start,
            month_end,
        );
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    // ── sort_calendar_items ──

    #[test]
    fn sort_calendar_items_by_date() {
        let uid = Uuid::new_v4();
        let mut items = vec![
            CalendarItem {
                subscription_id: uid,
                name: "B".to_string(),
                amount: Decimal::TEN,
                currency: "USD".to_string(),
                frequency: "weekly".to_string(),
                billing_date: NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
                is_active: true,
                logo_url: None,
            },
            CalendarItem {
                subscription_id: uid,
                name: "A".to_string(),
                amount: Decimal::TEN,
                currency: "USD".to_string(),
                frequency: "weekly".to_string(),
                billing_date: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
                is_active: true,
                logo_url: None,
            },
        ];
        sort_calendar_items(&mut items);
        assert!(items[0].billing_date < items[1].billing_date);
        assert_eq!(items[0].name, "A");
        assert_eq!(items[1].name, "B");
    }

    #[test]
    fn sort_calendar_items_empty() {
        let mut items: Vec<CalendarItem> = vec![];
        sort_calendar_items(&mut items);
        assert!(items.is_empty());
    }

    #[test]
    fn sort_calendar_items_single() {
        let uid = Uuid::new_v4();
        let mut items = vec![CalendarItem {
            subscription_id: uid,
            name: "Solo".to_string(),
            amount: Decimal::ONE,
            currency: "EUR".to_string(),
            frequency: "monthly".to_string(),
            billing_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            is_active: true,
            logo_url: None,
        }];
        sort_calendar_items(&mut items);
        assert_eq!(items.len(), 1);
    }

    // ── CalendarItem construction & billing loop logic ──

    #[test]
    fn calendar_item_construction_with_logo() {
        let uid = Uuid::new_v4();
        let item = CalendarItem {
            subscription_id: uid,
            name: "Netflix".to_string(),
            amount: Decimal::new(1599, 2),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            billing_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            is_active: true,
            logo_url: Some("https://logo.com/netflix.png".to_string()),
        };
        assert_eq!(item.name, "Netflix");
        assert_eq!(
            item.logo_url.as_deref(),
            Some("https://logo.com/netflix.png")
        );
    }

    #[test]
    fn calendar_item_construction_no_logo() {
        let uid = Uuid::new_v4();
        let item = CalendarItem {
            subscription_id: uid,
            name: "Spotify".to_string(),
            amount: Decimal::new(999, 2),
            currency: "EUR".to_string(),
            frequency: "monthly".to_string(),
            billing_date: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            is_active: true,
            logo_url: None,
        };
        assert!(item.logo_url.is_none());
    }

    #[test]
    fn calendar_item_inactive() {
        let uid = Uuid::new_v4();
        let item = CalendarItem {
            subscription_id: uid,
            name: "Old Sub".to_string(),
            amount: Decimal::ONE,
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            billing_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            is_active: false,
            logo_url: None,
        };
        assert!(!item.is_active);
    }

    #[test]
    fn calendar_item_serialize() {
        let uid = Uuid::new_v4();
        let item = CalendarItem {
            subscription_id: uid,
            name: "Test".to_string(),
            amount: Decimal::TEN,
            currency: "USD".to_string(),
            frequency: "weekly".to_string(),
            billing_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            is_active: true,
            logo_url: None,
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["name"], "Test");
        assert_eq!(json["currency"], "USD");
        assert_eq!(json["is_active"], true);
    }

    #[test]
    fn calendar_billing_loop_monthly_single_date() {
        let month_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "monthly",
            month_start,
            month_end,
        );
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    }

    #[test]
    fn calendar_billing_loop_weekly_five_dates_in_jan() {
        let month_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            "weekly",
            month_start,
            month_end,
        );
        // 1, 8, 15, 22, 29 = 5 dates
        assert_eq!(dates.len(), 5);
    }

    #[test]
    fn calendar_billing_loop_weekly_end_of_february_leap() {
        let month_start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 29).unwrap(),
            "weekly",
            month_start,
            month_end,
        );
        // Feb: 5, 12, 19, 26 = 4 dates
        assert_eq!(dates.len(), 4);
        for d in &dates {
            assert!(*d >= month_start && *d <= month_end);
        }
    }

    #[test]
    fn calendar_billing_loop_quarterly_in_month() {
        let month_start = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 4, 30).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "quarterly",
            month_start,
            month_end,
        );
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0].month(), 4);
    }

    #[test]
    fn calendar_billing_loop_yearly_in_month() {
        let month_start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            "yearly",
            month_start,
            month_end,
        );
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    }

    #[test]
    fn calendar_billing_loop_same_day_as_month_end() {
        let month_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            "monthly",
            month_start,
            month_end,
        );
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }

    #[test]
    fn calendar_billing_loop_february_short_month() {
        let month_start = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2023, 2, 28).unwrap();
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            "monthly",
            month_start,
            month_end,
        );
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0].day(), 28);
    }

    #[test]
    fn advance_date_monthly_crosses_year_boundary() {
        // Nov 15 + 1 month = Dec 15 (advance_date adds 1 month)
        let d = NaiveDate::from_ymd_opt(2024, 11, 15).unwrap();
        let next = advance_date(d, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2024, 12, 15).unwrap());
    }

    #[test]
    fn advance_date_monthly_from_jan_30_to_feb_28() {
        let d = NaiveDate::from_ymd_opt(2023, 1, 30).unwrap();
        let next = advance_date(d, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());
    }

    #[test]
    fn add_months_24_months_two_years() {
        let d = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = add_months(d, 24);
        assert_eq!(result, NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
    }

    #[test]
    fn add_months_jan_1_to_december() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let result = add_months(d, 11);
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
    }

    #[test]
    fn add_months_feb_29_leap_to_mar_29() {
        let d = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let result = add_months(d, 1);
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 3, 29).unwrap());
    }

    #[test]
    fn add_months_dec_15_to_jan_next_year() {
        let d = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
        let result = add_months(d, 1);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    }

    #[test]
    fn sort_calendar_items_multiple_dates_out_of_order() {
        let uid = Uuid::new_v4();
        let mut items = vec![
            CalendarItem {
                subscription_id: uid,
                name: "C".to_string(),
                amount: Decimal::TEN,
                currency: "USD".to_string(),
                frequency: "weekly".to_string(),
                billing_date: NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
                is_active: true,
                logo_url: None,
            },
            CalendarItem {
                subscription_id: uid,
                name: "A".to_string(),
                amount: Decimal::TEN,
                currency: "USD".to_string(),
                frequency: "weekly".to_string(),
                billing_date: NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                is_active: true,
                logo_url: None,
            },
            CalendarItem {
                subscription_id: uid,
                name: "B".to_string(),
                amount: Decimal::TEN,
                currency: "USD".to_string(),
                frequency: "weekly".to_string(),
                billing_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                is_active: true,
                logo_url: None,
            },
        ];
        sort_calendar_items(&mut items);
        assert_eq!(items[0].name, "A");
        assert_eq!(items[1].name, "B");
        assert_eq!(items[2].name, "C");
    }

    #[test]
    fn calculate_next_billing_date_quarterly_future() {
        let future = NaiveDate::from_ymd_opt(2099, 1, 15).unwrap();
        let result = calculate_next_billing_date(future, "quarterly");
        assert_eq!(result, future);
    }

    #[test]
    fn calculate_next_billing_date_past_quarterly_advances() {
        let past = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let result = calculate_next_billing_date(past, "quarterly");
        assert!(result > chrono::Utc::now().date_naive());
    }

    // ── advance_date all frequency branches ──

    #[test]
    fn advance_date_weekly() {
        let d = NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
        let next = advance_date(d, "weekly");
        assert_eq!(next, d + chrono::Duration::weeks(1));
    }

    #[test]
    fn advance_date_quarterly() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let next = advance_date(d, "quarterly");
        assert_eq!(next, add_months(d, 3));
    }

    #[test]
    fn advance_date_default_fallback() {
        let d = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let next = advance_date(d, "biweekly"); // unknown frequency
        assert_eq!(next, d + chrono::Duration::days(30)); // fallback
    }

    // ── add_months edge cases ──

    #[test]
    fn add_months_negative() {
        let d = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = add_months(d, -3);
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn add_months_large_positive() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let result = add_months(d, 120); // 10 years
        assert_eq!(result, NaiveDate::from_ymd_opt(2034, 1, 1).unwrap());
    }

    // ── calculate_next_billing_date edge cases ──

    #[test]
    fn next_billing_date_yesterday_advances_once() {
        let yesterday = chrono::Utc::now().date_naive() - chrono::Duration::days(1);
        let result = calculate_next_billing_date(yesterday, "monthly");
        assert!(result > yesterday);
    }

    #[test]
    fn next_billing_date_past_weekly() {
        let past = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let result = calculate_next_billing_date(past, "weekly");
        assert!(result >= chrono::Utc::now().date_naive());
    }

    // ── sort_calendar_items additional cases ──

    #[test]
    fn sort_calendar_items_equal_dates() {
        let uid1 = Uuid::new_v4();
        let uid2 = Uuid::new_v4();
        let mut items = vec![
            CalendarItem {
                subscription_id: uid2,
                name: "B".to_string(),
                amount: Decimal::ONE,
                currency: "USD".to_string(),
                frequency: "weekly".to_string(),
                billing_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                is_active: true,
                logo_url: None,
            },
            CalendarItem {
                subscription_id: uid1,
                name: "A".to_string(),
                amount: Decimal::ONE,
                currency: "USD".to_string(),
                frequency: "weekly".to_string(),
                billing_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                is_active: true,
                logo_url: None,
            },
        ];
        sort_calendar_items(&mut items);
        assert_eq!(items.len(), 2);
    }

    // ── generate_billing_dates additional cases ──

    #[test]
    fn generate_billing_dates_multiple_months_covered() {
        let month_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        // Jan 15 starting, monthly → Jan 15, Feb 15, Mar 15
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "monthly",
            month_start,
            month_end,
        );
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0].month(), 1);
        assert_eq!(dates[1].month(), 2);
        assert_eq!(dates[2].month(), 3);
    }

    #[test]
    fn generate_billing_dates_before_month_start_advances_multiple() {
        let month_start = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 4, 30).unwrap();
        // Jan 15 → Apr 15 (advances past Feb and Mar)
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "monthly",
            month_start,
            month_end,
        );
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2024, 4, 15).unwrap());
    }

    #[test]
    fn generate_billing_dates_yearly_no_match() {
        let month_start = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
        let month_end = NaiveDate::from_ymd_opt(2024, 7, 31).unwrap();
        // Yearly billing in Jan → no occurrence in July
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "yearly",
            month_start,
            month_end,
        );
        assert!(dates.is_empty());
    }

    #[test]
    fn add_months_neg() {
        let d = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(
            add_months(d, 6),
            NaiveDate::from_ymd_opt(2024, 12, 15).unwrap()
        );
    }

    #[test]
    fn add_months_leap() {
        let d = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        assert_eq!(
            add_months(d, 1),
            NaiveDate::from_ymd_opt(2024, 3, 29).unwrap()
        );
    }

    #[test]
    fn add_months_clamp() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        assert_eq!(
            add_months(d, 1),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
    }

    #[test]
    fn add_months_q() {
        let d = NaiveDate::from_ymd_opt(2024, 11, 15).unwrap();
        assert_eq!(
            add_months(d, 3),
            NaiveDate::from_ymd_opt(2025, 2, 15).unwrap()
        );
    }

    #[test]
    fn adv_unknown() {
        let d = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(advance_date(d, "biweekly"), d + chrono::Duration::days(30));
    }

    #[test]
    fn adv_leap() {
        let d = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        assert_eq!(
            advance_date(d, "yearly"),
            NaiveDate::from_ymd_opt(2025, 2, 28).unwrap()
        );
    }

    #[test]
    fn cal_ser() {
        let uid = Uuid::new_v4();
        let item = CalendarItem {
            subscription_id: uid,
            name: "T".to_string(),
            amount: Decimal::TEN,
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            billing_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            is_active: true,
            logo_url: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("T"));
    }

    #[test]
    fn gbds_weekly() {
        let dates = generate_billing_dates(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            "weekly",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );
        assert_eq!(dates.len(), 5);
    }

    #[test]
    fn cnxt_weekly() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let r = calculate_next_billing_date(d, "weekly");
        assert!(r > d);
    }

    #[test]
    fn sub_svc() {
        let _ = SubscriptionService;
    }
}
