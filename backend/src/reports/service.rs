use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::HashMap;
use chrono::Datelike;

use crate::reports::models::{CategoryBreakdown, MonthlySummary, ReportResponse, SpendingTrend};

pub struct ReportService;

impl ReportService {
    pub async fn get_monthly_summary(
        pool: &PgPool,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> MonthlySummary {
        let from_date = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let to_date = if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
        };

        // Get subscriptions total (normalized to monthly)
        let subscriptions: Vec<(Decimal,)> = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(
                CASE frequency
                    WHEN 'weekly' THEN amount * 4.33
                    WHEN 'monthly' THEN amount
                    WHEN 'quarterly' THEN amount / 3
                    WHEN 'yearly' THEN amount / 12
                    ELSE amount
                END
            ), 0) as total
            FROM subscriptions 
            WHERE user_id = $1 AND is_active = true
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let subscriptions_total = subscriptions.first().map(|r| r.0).unwrap_or(Decimal::ZERO);

        // Get transactions total for the month
        let transactions: Vec<(Decimal, i64)> = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(amount), 0) as total, COUNT(*) as count
            FROM transactions 
            WHERE user_id = $1 AND date >= $2 AND date <= $3
            "#,
        )
        .bind(user_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let (transactions_total, transactions_count) = transactions
            .first()
            .map(|r| (r.0, r.1))
            .unwrap_or((Decimal::ZERO, 0));

        // Count active subscriptions
        let subscription_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM subscriptions WHERE user_id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        MonthlySummary {
            year,
            month,
            subscriptions_total,
            transactions_total,
            overall_total: subscriptions_total + transactions_total,
            subscriptions_count: subscription_count.0,
            transactions_count,
        }
    }

    pub async fn get_category_breakdown(
        pool: &PgPool,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> Vec<CategoryBreakdown> {
        let from_date = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let to_date = if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
        };

        // Get subscriptions by category
        let subs_by_cat: Vec<(Option<String>, String, Decimal)> = sqlx::query_as(
            r#"
            SELECT c.id::text, COALESCE(c.name, 'Uncategorized'),
                   COALESCE(SUM(
                       CASE s.frequency
                           WHEN 'weekly' THEN s.amount * 4.33
                           WHEN 'monthly' THEN s.amount
                           WHEN 'quarterly' THEN s.amount / 3
                           WHEN 'yearly' THEN s.amount / 12
                           ELSE s.amount
                       END
                   ), 0) as total
            FROM subscriptions s
            LEFT JOIN categories c ON s.category_id = c.id
            WHERE s.user_id = $1 AND s.is_active = true
            GROUP BY c.id, c.name
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        // Get transactions by category
        let txns_by_cat: Vec<(Option<String>, String, Decimal)> = sqlx::query_as(
            r#"
            SELECT c.id::text, COALESCE(c.name, 'Uncategorized'), COALESCE(SUM(t.amount), 0) as total
            FROM transactions t
            LEFT JOIN categories c ON t.category_id = c.id
            WHERE t.user_id = $1 AND t.date >= $2 AND t.date <= $3
            GROUP BY c.id, c.name
            "#,
        )
        .bind(user_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        // Combine and aggregate
        let mut category_totals: HashMap<String, Decimal> = HashMap::new();
        let mut category_ids: HashMap<String, Option<String>> = HashMap::new();
        let mut category_counts: HashMap<String, i64> = HashMap::new();

        for (cat_id, cat_name, total) in subs_by_cat {
            let key = cat_name.clone();
            *category_totals.entry(key.clone()).or_insert(Decimal::ZERO) += total;
            category_ids.entry(key.clone()).or_insert(cat_id);
            *category_counts.entry(key).or_insert(0) += 1;
        }

        for (cat_id, cat_name, total) in txns_by_cat {
            let key = cat_name.clone();
            *category_totals.entry(key.clone()).or_insert(Decimal::ZERO) += total;
            if !category_ids.contains_key(&key) {
                category_ids.insert(key.clone(), cat_id);
            }
            *category_counts.entry(key).or_insert(0) += 1;
        }

        let overall_total: Decimal = category_totals.values().sum();
        let mut breakdown: Vec<CategoryBreakdown> = category_totals
            .into_iter()
            .map(|(name, total)| {
                let percentage = if overall_total > Decimal::ZERO {
                    let pct = (total / overall_total * Decimal::from(100)).round_dp(1);
                    pct.to_string().parse().unwrap_or(0.0)
                } else {
                    0.0
                };
                CategoryBreakdown {
                    category_id: category_ids.remove(&name).unwrap_or(None),
                    category_name: name.clone(),
                    total,
                    count: *category_counts.get(&name).unwrap_or(&0),
                    percentage,
                }
            })
            .collect();

        breakdown.sort_by(|a, b| b.total.cmp(&a.total));
        breakdown
    }

    pub async fn get_recent_trends(
        pool: &PgPool,
        user_id: Uuid,
        months_count: i32,
    ) -> Vec<SpendingTrend> {
        let mut trends = Vec::new();

        for i in 0..months_count {
            let target_date = subtract_months_from_date(chrono::Utc::now().date_naive(), i);
            let year = target_date.year();
            let month = target_date.month();

            let summary = Self::get_monthly_summary(pool, user_id, year, month).await;

            trends.push(SpendingTrend {
                period: format!("{}/{:02}", month, year % 100),
                total: summary.overall_total,
            });
        }

        trends
    }

    pub async fn get_full_report(
        pool: &PgPool,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> ReportResponse {
        let monthly_summary = Self::get_monthly_summary(pool, user_id, year, month).await;
        let category_breakdown = Self::get_category_breakdown(pool, user_id, year, month).await;
        let recent_trends = Self::get_recent_trends(pool, user_id, 6).await;

        ReportResponse {
            monthly_summary,
            category_breakdown,
            recent_trends,
        }
    }
}

fn subtract_months_from_date(date: chrono::NaiveDate, months: i32) -> chrono::NaiveDate {
    let year = date.year();
    let month = date.month();
    let day = date.day();
    let total_months = year * 12 + month as i32 - 1 - months;
    let new_year = total_months / 12;
    let new_month = ((total_months % 12) + 1) as u32;
    let max_day = days_in_month(new_year, new_month);
    let new_day = day.min(max_day);
    chrono::NaiveDate::from_ymd_opt(new_year, new_month, new_day).unwrap_or(date)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 }
        }
        _ => 30,
    }
}
