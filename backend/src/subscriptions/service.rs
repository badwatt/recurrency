use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::date_utils::days_in_month;
use crate::shared::error::{AppError, AppResult};
use crate::subscriptions::models::{
    CreateSubscriptionRequest, Subscription, UpdateSubscriptionRequest,
};
use crate::categories::service::CategoryService;

pub struct SubscriptionService;

impl SubscriptionService {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        req: CreateSubscriptionRequest,
    ) -> AppResult<Subscription> {
        let next_billing_date = calculate_next_billing_date(req.start_date, &req.frequency);

        // Auto-assign default category if none provided
        let category_id = match req.category_id {
            Some(id) => id,
            None => CategoryService::get_default(pool, user_id).await?.id,
        };

        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            INSERT INTO subscriptions (id, user_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10, $11, $12, $12)
            RETURNING id, user_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
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
            "SELECT id, user_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at FROM subscriptions WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(subscriptions)
    }

    pub async fn get_by_id(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Subscription> {
        let subscription = sqlx::query_as::<_, Subscription>(
            "SELECT id, user_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at FROM subscriptions WHERE id = $1 AND user_id = $2"
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
        let frequency = req.frequency.unwrap_or(existing.frequency.clone());
        let start_date = req.start_date.unwrap_or(existing.start_date);
        let next_billing_date = req.next_billing_date
            .unwrap_or_else(|| calculate_next_billing_date(start_date, &frequency));
        let category_id = req.category_id.unwrap_or(existing.category_id);
        let is_active = req.is_active.unwrap_or(existing.is_active);
        let notes = req.notes.or(existing.notes);
        let logo_url = req.logo_url.or(existing.logo_url);

        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            UPDATE subscriptions 
            SET name = $1, amount = $2, currency = $3, frequency = $4, start_date = $5, 
                next_billing_date = $6, category_id = $7, is_active = $8, notes = $9, logo_url = $10, updated_at = $11
            WHERE id = $12 AND user_id = $13
            RETURNING id, user_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(amount)
        .bind(&currency)
        .bind(&frequency)
        .bind(start_date)
        .bind(next_billing_date)
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

    pub async fn get_upcoming(pool: &PgPool, user_id: Uuid, days: i64) -> AppResult<Vec<Subscription>> {
        let from_date = Utc::now().date_naive();
        let to_date = from_date + chrono::Duration::days(days);

        let subscriptions = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT id, user_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at 
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
            "SELECT id, user_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active, notes, logo_url, created_at, updated_at FROM subscriptions WHERE user_id = $1 AND is_active = true"
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
            // Calculate all billing dates within the month
            let mut billing_date = sub.next_billing_date.unwrap_or(sub.start_date);
            
            // If billing date is before month start, advance to next occurrence
            while billing_date < month_start {
                billing_date = advance_date(billing_date, &sub.frequency);
            }

            // Add all billing dates within the month
            while billing_date <= month_end {
                calendar_items.push(CalendarItem {
                    date: billing_date,
                    subscription_id: sub.id,
                    name: sub.name.clone(),
                    amount: sub.amount,
                    currency: sub.currency.clone(),
                    frequency: sub.frequency.clone(),
                    logo_url: sub.logo_url.clone(),
                });
                billing_date = advance_date(billing_date, &sub.frequency);
            }
        }

        // Sort by date
        calendar_items.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(calendar_items)
    }
}

#[derive(Debug, Serialize)]
pub struct CalendarItem {
    pub date: NaiveDate,
    pub subscription_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub logo_url: Option<String>,
}

fn advance_date(date: NaiveDate, frequency: &str) -> NaiveDate {
    match frequency {
        "weekly" => date + chrono::Duration::weeks(1),
        "monthly" => add_months(date, 1),
        "quarterly" => add_months(date, 3),
        "yearly" => add_months(date, 12),
        _ => date + chrono::Duration::days(30),
    }
}

fn calculate_next_billing_date(start_date: NaiveDate, frequency: &str) -> NaiveDate {
    let today = Utc::now().date_naive();
    let mut next_date = start_date;

    while next_date <= today {
        next_date = match frequency {
            "weekly" => next_date + chrono::Duration::weeks(1),
            "monthly" => add_months(next_date, 1),
            "quarterly" => add_months(next_date, 3),
            "yearly" => add_months(next_date, 12),
            _ => next_date + chrono::Duration::days(30),
        };
    }

    next_date
}

fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
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
