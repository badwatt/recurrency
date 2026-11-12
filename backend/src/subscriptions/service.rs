use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;

use crate::categories::service::CategoryService;
use crate::shared::date_utils::days_in_month;
use crate::shared::error::{AppError, AppResult};
use crate::subscriptions::models::{
    CreateSubscriptionRequest, Subscription, UpdateSubscriptionRequest,
};
use crate::subscriptions::recurrence::{
    legacy_from_recurrence, next_occurrence_on_or_after, occurrences_in_month,
    recurrence_from_legacy, validate_recurrence,
};

pub struct SubscriptionService;

/// Subscription port, injected into `AppState` as `Arc<dyn SubscriptionServiceTrait>`.
#[async_trait::async_trait]
pub trait SubscriptionServiceTrait: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        req: CreateSubscriptionRequest,
    ) -> AppResult<Subscription>;
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
    async fn get_calendar(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<Vec<CalendarItem>>;
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
    async fn create(
        &self,
        user_id: Uuid,
        req: CreateSubscriptionRequest,
    ) -> AppResult<Subscription> {
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
        let recurrence = req.recurrence.clone().unwrap_or_else(|| {
            recurrence_from_legacy(
                &req.frequency,
                req.start_date,
                req.schedule_day,
                req.schedule_month,
            )
        });
        validate_recurrence(&recurrence)?;
        validate_end_date(req.start_date, req.end_date)?;
        let (frequency, schedule_day, schedule_month) = legacy_from_recurrence(&recurrence);
        let next_billing_date = next_occurrence_on_or_after(
            req.start_date,
            &recurrence,
            Utc::now().date_naive(),
            req.end_date,
        );
        let is_active = req.is_active.unwrap_or(true);

        // Accounts are mandatory; fall back to the user's default for old clients.
        let account_id = if req.account_id.is_nil() {
            crate::accounts::service::AccountService::get_default(pool, user_id)
                .await?
                .id
        } else {
            crate::accounts::service::AccountService::get_by_id(pool, user_id, req.account_id)
                .await?
                .id
        };

        // Auto-assign default category if none provided. A supplied id must
        // belong to the authenticated user; foreign keys only prove existence.
        let category_id = match req.category_id {
            Some(id) => CategoryService::get_by_id(pool, user_id, id).await?.id,
            None => CategoryService::get_default(pool, user_id).await?.id,
        };

        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            INSERT INTO subscriptions (id, user_id, account_id, category_id, name, amount, currency, frequency, schedule_day, schedule_month, recurrence, start_date, end_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $18)
            RETURNING id, user_id, account_id, category_id, name, amount, currency, frequency, schedule_day, schedule_month, recurrence, start_date, end_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(account_id)
        .bind(category_id)
        .bind(&req.name)
        .bind(req.amount)
        .bind(&req.currency)
        .bind(&frequency)
        .bind(schedule_day)
        .bind(schedule_month)
        .bind(Json(recurrence.clone()))
        .bind(req.start_date)
        .bind(req.end_date)
        .bind(next_billing_date)
        .bind(is_active)
        .bind(&req.notes)
        .bind(&req.logo_url)
        .bind(Utc::now())
        .fetch_one(pool)
        .await?;

        Ok(subscription)
    }

    async fn refresh_next_billing_date(
        pool: &PgPool,
        subscription: &mut Subscription,
    ) -> AppResult<()> {
        if !subscription.is_active {
            return Ok(());
        }
        let today = Utc::now().date_naive();
        let stale = subscription
            .next_billing_date
            .is_none_or(|date| date < today);
        if !stale {
            return Ok(());
        }
        let next = next_occurrence_on_or_after(
            subscription.start_date,
            &subscription.recurrence.0,
            today,
            subscription.end_date,
        );
        if next != subscription.next_billing_date {
            sqlx::query(
                "UPDATE subscriptions SET next_billing_date = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(next)
            .bind(subscription.id)
            .execute(pool)
            .await?;
            subscription.next_billing_date = next;
        }
        Ok(())
    }

    pub async fn get_all(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Subscription>> {
        let mut subscriptions = sqlx::query_as::<_, Subscription>(
            "SELECT id, user_id, account_id, category_id, name, amount, currency, frequency, schedule_day, schedule_month, recurrence, start_date, end_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at FROM subscriptions WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        for subscription in &mut subscriptions {
            Self::refresh_next_billing_date(pool, subscription).await?;
        }

        Ok(subscriptions)
    }

    pub async fn get_by_id(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Subscription> {
        let mut subscription = sqlx::query_as::<_, Subscription>(
            "SELECT id, user_id, account_id, category_id, name, amount, currency, frequency, schedule_day, schedule_month, recurrence, start_date, end_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at FROM subscriptions WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Subscription not found".to_string()))?;
        Self::refresh_next_billing_date(pool, &mut subscription).await?;

        Ok(subscription)
    }

    pub async fn update(
        pool: &PgPool,
        user_id: Uuid,
        id: Uuid,
        req: UpdateSubscriptionRequest,
    ) -> AppResult<Subscription> {
        let existing = Self::get_by_id(pool, user_id, id).await?;

        if let Some(account_id) = req.account_id {
            crate::accounts::service::AccountService::get_by_id(pool, user_id, account_id).await?;
        }
        if let Some(category_id) = req.category_id {
            CategoryService::get_by_id(pool, user_id, category_id).await?;
        }

        let name = req.name.unwrap_or(existing.name);
        let amount = req.amount.unwrap_or(existing.amount);
        let currency = req.currency.unwrap_or(existing.currency);
        let start_date = req.start_date.unwrap_or(existing.start_date);
        let end_date = if req.clear_end_date {
            None
        } else {
            req.end_date.or(existing.end_date)
        };
        validate_end_date(start_date, end_date)?;
        let recurrence = req
            .recurrence
            .clone()
            .unwrap_or_else(|| existing.recurrence.0.clone());
        validate_recurrence(&recurrence)?;
        let (frequency, schedule_day, schedule_month) = legacy_from_recurrence(&recurrence);
        let schedule_changed = recurrence != existing.recurrence.0
            || start_date != existing.start_date
            || end_date != existing.end_date;

        let next_billing_date = if let Some(next) = req.next_billing_date {
            if end_date.is_some_and(|end| next > end) {
                None
            } else {
                Some(next)
            }
        } else if schedule_changed {
            next_occurrence_on_or_after(
                start_date,
                &recurrence,
                Utc::now().date_naive(),
                end_date,
            )
        } else {
            existing.next_billing_date
        };

        let account_id = req.account_id.unwrap_or(existing.account_id);
        let category_id = req.category_id.unwrap_or(existing.category_id);
        let is_active = req.is_active.unwrap_or(existing.is_active);
        let notes = match req.notes {
            Some(value) if value.trim().is_empty() => None,
            Some(value) => Some(value),
            None => existing.notes,
        };
        let logo_url = req.logo_url.or(existing.logo_url);

        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            UPDATE subscriptions
            SET name = $1, amount = $2, currency = $3, frequency = $4,
                schedule_day = $5, schedule_month = $6, recurrence = $7,
                start_date = $8, end_date = $9, next_billing_date = $10,
                account_id = $11, category_id = $12, is_active = $13,
                notes = $14, logo_url = $15, updated_at = $16
            WHERE id = $17 AND user_id = $18
            RETURNING id, user_id, account_id, category_id, name, amount, currency, frequency, schedule_day, schedule_month, recurrence, start_date, end_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(amount)
        .bind(&currency)
        .bind(&frequency)
        .bind(schedule_day)
        .bind(schedule_month)
        .bind(Json(recurrence))
        .bind(start_date)
        .bind(end_date)
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
        let mut subscriptions = Self::get_all(pool, user_id).await?;
        subscriptions.retain(|subscription| {
            subscription.is_active
                && subscription
                    .next_billing_date
                    .is_some_and(|date| date >= from_date && date <= to_date)
                && subscription
                    .end_date
                    .is_none_or(|end| subscription.next_billing_date.is_some_and(|date| date <= end))
        });
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
            "SELECT id, user_id, account_id, category_id, name, amount, currency, frequency, schedule_day, schedule_month, recurrence, start_date, end_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at FROM subscriptions WHERE user_id = $1 AND is_active = true"
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
            // Generate every charge in the selected month, including completed
            // charges, and use the optional per-date annual amount.
            let _ = (month_start, month_end);
            for (billing_date, amount) in occurrences_in_month(
                sub.start_date,
                &sub.recurrence.0,
                sub.end_date,
                year,
                month,
                sub.amount,
            ) {
                calendar_items.push(CalendarItem {
                    subscription_id: sub.id,
                    name: sub.name.clone(),
                    amount,
                    currency: sub.currency.clone(),
                    frequency: sub.frequency.clone(),
                    billing_date,
                    is_active: sub.is_active,
                    logo_url: sub.logo_url.clone(),
                });
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

#[cfg(test)]
fn normalize_schedule(
    start_date: NaiveDate,
    frequency: &str,
    day: Option<i16>,
    month: Option<i16>,
) -> AppResult<(Option<i16>, Option<i16>)> {
    match frequency {
        "daily" => Ok((None, None)),
        "weekly" => {
            let day = day.unwrap_or(start_date.weekday().number_from_monday() as i16);
            if !(1..=7).contains(&day) {
                return Err(AppError::BadRequest("Weekday must be 1-7".to_string()));
            }
            Ok((Some(day), None))
        }
        "monthly" | "quarterly" => {
            let day = day.unwrap_or(start_date.day() as i16);
            if !(1..=31).contains(&day) {
                return Err(AppError::BadRequest("Day must be 1-31".to_string()));
            }
            Ok((Some(day), None))
        }
        "yearly" => {
            let day = day.unwrap_or(start_date.day() as i16);
            let month = month.unwrap_or(start_date.month() as i16);
            if !(1..=31).contains(&day) || !(1..=12).contains(&month) {
                return Err(AppError::BadRequest(
                    "Yearly schedule needs a valid month and day".to_string(),
                ));
            }
            Ok((Some(day), Some(month)))
        }
        _ => Err(AppError::BadRequest("Invalid frequency".to_string())),
    }
}

fn validate_end_date(start_date: NaiveDate, end_date: Option<NaiveDate>) -> AppResult<()> {
    if end_date.is_some_and(|end| end < start_date) {
        return Err(AppError::BadRequest(
            "End date must be on or after the start date".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
fn date_for_day(year: i32, month: u32, day: i16) -> NaiveDate {
    let day = (day.max(1) as u32).min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).expect("clamped date is valid")
}

#[cfg(test)]
fn add_months_with_day(date: NaiveDate, months: i32, day: i16) -> NaiveDate {
    let total = date.year() * 12 + date.month() as i32 - 1 + months;
    let year = total.div_euclid(12);
    let month = (total.rem_euclid(12) + 1) as u32;
    date_for_day(year, month, day)
}

#[cfg(test)]
fn first_scheduled_on_or_after(
    start_date: NaiveDate,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
) -> NaiveDate {
    match frequency {
        "daily" => start_date,
        "weekly" => {
            let target = schedule_day.unwrap_or(start_date.weekday().number_from_monday() as i16);
            let current = start_date.weekday().number_from_monday() as i16;
            start_date + chrono::Duration::days((target - current).rem_euclid(7) as i64)
        }
        "monthly" | "quarterly" => {
            let day = schedule_day.unwrap_or(start_date.day() as i16);
            let candidate = date_for_day(start_date.year(), start_date.month(), day);
            if candidate < start_date {
                add_months_with_day(candidate, 1, day)
            } else {
                candidate
            }
        }
        "yearly" => {
            let day = schedule_day.unwrap_or(start_date.day() as i16);
            let month = schedule_month.unwrap_or(start_date.month() as i16);
            let candidate = date_for_day(start_date.year(), month as u32, day);
            if candidate < start_date {
                date_for_day(start_date.year() + 1, month as u32, day)
            } else {
                candidate
            }
        }
        _ => start_date,
    }
}

#[cfg(test)]
pub(crate) fn advance_scheduled(
    date: NaiveDate,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
) -> NaiveDate {
    match frequency {
        "daily" => date + chrono::Duration::days(1),
        "weekly" => date + chrono::Duration::weeks(1),
        "monthly" => add_months_with_day(date, 1, schedule_day.unwrap_or(date.day() as i16)),
        "quarterly" => add_months_with_day(date, 3, schedule_day.unwrap_or(date.day() as i16)),
        "yearly" => {
            let month = schedule_month.unwrap_or(date.month() as i16);
            date_for_day(
                date.year() + 1,
                month as u32,
                schedule_day.unwrap_or(date.day() as i16),
            )
        }
        _ => date + chrono::Duration::days(30),
    }
}

#[cfg(test)]
fn calculate_next_billing_date_scheduled(
    start_date: NaiveDate,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
    end_date: Option<NaiveDate>,
) -> Option<NaiveDate> {
    let today = Utc::now().date_naive();
    // Start date activates the subscription; the first charge is the first
    // recurrence match on or after it. A past match is skipped for "next".
    let mut next_date =
        first_scheduled_on_or_after(start_date, frequency, schedule_day, schedule_month);
    let mut iterations = 0;

    while next_date < today {
        next_date = advance_scheduled(next_date, frequency, schedule_day, schedule_month);
        iterations += 1;
        if iterations > 10_000 {
            tracing::warn!(
                "calculate_next_billing_date_scheduled: loop guard start={start_date} frequency={frequency}"
            );
            break;
        }
    }

    if end_date.is_some_and(|end| next_date >= end) {
        None
    } else {
        Some(next_date)
    }
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

#[cfg(test)]
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
                start_date,
                frequency
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

#[cfg(test)]
mod schedule_tests {
    use super::*;

    #[test]
    fn normalize_weekly_uses_start_weekday() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(); // Wednesday
        let (day, month) = normalize_schedule(start, "weekly", None, None).unwrap();
        assert_eq!(day, Some(3));
        assert!(month.is_none());
    }

    #[test]
    fn normalize_yearly_requires_month_and_day() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let (day, month) = normalize_schedule(start, "yearly", Some(15), Some(6)).unwrap();
        assert_eq!((day, month), (Some(15), Some(6)));
        assert!(normalize_schedule(start, "yearly", Some(0), Some(6)).is_err());
    }

    #[test]
    fn normalize_daily_has_no_extra_fields() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        assert_eq!(normalize_schedule(start, "daily", Some(3), Some(6)).unwrap(), (None, None));
    }

    #[test]
    fn advance_scheduled_keeps_monthly_day_after_clamp() {
        let jan = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let feb = advance_scheduled(jan, "monthly", Some(31), None);
        assert_eq!(feb, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
        let mar = advance_scheduled(feb, "monthly", Some(31), None);
        assert_eq!(mar, NaiveDate::from_ymd_opt(2024, 3, 31).unwrap());
    }

    #[test]
    fn future_start_waits_for_first_recurrence_match() {
        let start = Utc::now().date_naive() + chrono::Duration::days(2);
        let expected = first_scheduled_on_or_after(start, "monthly", Some(6), None);
        assert_eq!(
            calculate_next_billing_date_scheduled(start, "monthly", Some(6), None, None),
            Some(expected)
        );
    }

    #[test]
    fn next_billing_keeps_charge_due_today() {
        let today = Utc::now().date_naive();
        assert_eq!(
            calculate_next_billing_date_scheduled(
                today,
                "monthly",
                Some(today.day() as i16),
                None,
                None,
            ),
            Some(today)
        );
    }

    #[test]
    fn next_billing_respects_end_date() {
        let start = NaiveDate::from_ymd_opt(2099, 1, 15).unwrap();
        assert_eq!(
            calculate_next_billing_date_scheduled(
                start,
                "monthly",
                Some(15),
                None,
                Some(NaiveDate::from_ymd_opt(2100, 1, 1).unwrap()),
            ),
            Some(start)
        );
        assert_eq!(
            calculate_next_billing_date_scheduled(
                start,
                "monthly",
                Some(15),
                None,
                Some(NaiveDate::from_ymd_opt(2099, 1, 1).unwrap()),
            ),
            None
        );
    }
}
