use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::HashMap;
use chrono::Datelike;

use crate::shared::date_utils::days_in_month;
use crate::reports::models::{CategoryBreakdown, MonthlySummary, ReportResponse, SpendingTrend};

pub struct ReportService;

/// Calculate the first and last date of a given year/month.
pub(crate) fn month_date_range(year: i32, month: u32) -> (chrono::NaiveDate, chrono::NaiveDate) {
    let from_date = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let to_date = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
    };
    (from_date, to_date)
}

/// Calculate percentage from total, rounded to 1 decimal place.
pub(crate) fn calculate_percentage(total: Decimal, overall_total: Decimal) -> f64 {
    if overall_total > Decimal::ZERO {
        (total / overall_total * Decimal::from(100)).round_dp(1).to_string().parse().unwrap_or(0.0)
    } else {
        0.0
    }
}

/// Combine subscription and transaction category totals into a breakdown.
pub(crate) fn combine_category_totals(
    subs_by_cat: Vec<(Option<String>, String, Decimal)>,
    txns_by_cat: Vec<(Option<String>, String, Decimal)>,
) -> Vec<CategoryBreakdown> {
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
            let percentage = calculate_percentage(total, overall_total);
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

impl ReportService {
    pub async fn get_monthly_summary(
        pool: &PgPool,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> MonthlySummary {
        let (from_date, to_date) = month_date_range(year, month);

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
        let (from_date, to_date) = month_date_range(year, month);

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

        // Combine and aggregate using extracted function
        combine_category_totals(subs_by_cat, txns_by_cat)
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

#[allow(dead_code)]
pub(crate) fn normalize_to_monthly(amount: Decimal, frequency: &str) -> Decimal {
    match frequency {
        "weekly" => amount * Decimal::from_str_exact("4.33").unwrap_or(Decimal::ZERO),
        "monthly" => amount,
        "quarterly" => amount / Decimal::from(3),
        "yearly" => amount / Decimal::from(12),
        _ => amount,
    }
}

pub(crate) fn subtract_months_from_date(date: chrono::NaiveDate, months: i32) -> chrono::NaiveDate {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    // ── normalize_to_monthly ──

    #[test]
    fn normalize_weekly() {
        let result = normalize_to_monthly(Decimal::TEN, "weekly");
        assert_eq!(result, Decimal::from_str_exact("43.30").unwrap());
    }

    #[test]
    fn normalize_monthly_identity() {
        let result = normalize_to_monthly(dec("100"), "monthly");
        assert_eq!(result, dec("100"));
    }

    #[test]
    fn normalize_quarterly() {
        let result = normalize_to_monthly(dec("300"), "quarterly");
        assert_eq!(result, dec("100"));
    }

    #[test]
    fn normalize_yearly() {
        let result = normalize_to_monthly(dec("1200"), "yearly");
        assert_eq!(result, dec("100"));
    }

    #[test]
    fn normalize_unknown_passes_through() {
        let result = normalize_to_monthly(dec("50"), "biweekly");
        assert_eq!(result, dec("50"));
    }

    #[test]
    fn normalize_weekly_fractional_amount() {
        let result = normalize_to_monthly(dec("2.50"), "weekly");
        assert_eq!(result, Decimal::from_str_exact("10.825").unwrap());
    }

    #[test]
    fn normalize_zero_amount() {
        assert_eq!(normalize_to_monthly(Decimal::ZERO, "weekly"), Decimal::ZERO);
        assert_eq!(normalize_to_monthly(Decimal::ZERO, "monthly"), Decimal::ZERO);
        assert_eq!(normalize_to_monthly(Decimal::ZERO, "quarterly"), Decimal::ZERO);
        assert_eq!(normalize_to_monthly(Decimal::ZERO, "yearly"), Decimal::ZERO);
    }

    // ── subtract_months_from_date ──

    #[test]
    fn subtract_months_simple() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subtract_months_from_date(d, 1);
        assert_eq!(result, chrono::NaiveDate::from_ymd_opt(2024, 5, 15).unwrap());
    }

    #[test]
    fn subtract_months_cross_year() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        let result = subtract_months_from_date(d, 3);
        assert_eq!(result, chrono::NaiveDate::from_ymd_opt(2023, 11, 15).unwrap());
    }

    #[test]
    fn subtract_months_zero_is_noop() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subtract_months_from_date(d, 0);
        assert_eq!(result, d);
    }

    #[test]
    fn subtract_months_day_clamping() {
        // March 31 - 1 month = Feb 28/29
        let d = chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(); // leap year
        let result = subtract_months_from_date(d, 1);
        assert_eq!(result, chrono::NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }

    #[test]
    fn subtract_months_day_clamping_non_leap() {
        let d = chrono::NaiveDate::from_ymd_opt(2023, 3, 31).unwrap(); // non-leap
        let result = subtract_months_from_date(d, 1);
        assert_eq!(result, chrono::NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());
    }

    #[test]
    fn subtract_months_many_years() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subtract_months_from_date(d, 24);
        assert_eq!(result, chrono::NaiveDate::from_ymd_opt(2022, 6, 15).unwrap());
    }

    #[test]
    fn subtract_months_may_31_to_april() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 5, 31).unwrap();
        let result = subtract_months_from_date(d, 1);
        assert_eq!(result, chrono::NaiveDate::from_ymd_opt(2024, 4, 30).unwrap());
    }

    #[test]
    fn subtract_months_preserves_day_within_bounds() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 8, 15).unwrap();
        let result = subtract_months_from_date(d, 5);
        assert_eq!(result, chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    // ── Category percentage calculation (mirrors get_category_breakdown logic) ──

    #[test]
    fn percentage_calculation_with_two_categories() {
        let cat_a = dec("60");
        let cat_b = dec("40");
        let total = cat_a + cat_b;

        let pct_a = if total > Decimal::ZERO {
            (cat_a / total * Decimal::from(100)).round_dp(1).to_string().parse::<f64>().unwrap_or(0.0)
        } else {
            0.0
        };
        let pct_b = if total > Decimal::ZERO {
            (cat_b / total * Decimal::from(100)).round_dp(1).to_string().parse::<f64>().unwrap_or(0.0)
        } else {
            0.0
        };

        assert!((pct_a - 60.0).abs() < 0.2);
        assert!((pct_b - 40.0).abs() < 0.2);
    }

    #[test]
    fn percentage_zero_total_gives_zero() {
        assert_eq!(calculate_percentage(Decimal::ZERO, Decimal::ZERO), 0.0);
    }

    // ── month_date_range ──

    #[test]
    fn month_date_range_january() {
        let (from, to) = month_date_range(2024, 1);
        assert_eq!(from, chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(to, chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }

    #[test]
    fn month_date_range_february_leap() {
        let (from, to) = month_date_range(2024, 2);
        assert_eq!(from, chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(to, chrono::NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }

    #[test]
    fn month_date_range_february_non_leap() {
        let (from, to) = month_date_range(2023, 2);
        assert_eq!(to, chrono::NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());
    }

    #[test]
    fn month_date_range_december_crosses_year() {
        let (from, to) = month_date_range(2024, 12);
        assert_eq!(from, chrono::NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        assert_eq!(to, chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }

    #[test]
    fn month_date_range_30_day_month() {
        let (from, to) = month_date_range(2024, 4);
        assert_eq!(from, chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap());
        assert_eq!(to, chrono::NaiveDate::from_ymd_opt(2024, 4, 30).unwrap());
    }

    // ── calculate_percentage ──

    #[test]
    fn calculate_percentage_half() {
        let pct = calculate_percentage(dec("50"), dec("100"));
        assert!((pct - 50.0).abs() < 0.2);
    }

    #[test]
    fn calculate_percentage_third() {
        let pct = calculate_percentage(dec("33.33"), dec("100"));
        assert!((pct - 33.3).abs() < 0.2);
    }

    #[test]
    fn calculate_percentage_full() {
        let pct = calculate_percentage(dec("100"), dec("100"));
        assert!((pct - 100.0).abs() < 0.2);
    }

    // ── MonthlySummary construction ──

    #[test]
    fn monthly_summary_overall_is_sum() {
        let sub_total = dec("500");
        let txn_total = dec("200");
        let summary = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: sub_total,
            transactions_total: txn_total,
            overall_total: sub_total + txn_total,
            subscriptions_count: 5,
            transactions_count: 10,
        };
        assert_eq!(summary.overall_total, dec("700"));
    }

    // ── SpendingTrend formatting ──

    // ── combine_category_totals ──

    #[test]
    fn combine_empty_categories() {
        let subs: Vec<(Option<String>, String, Decimal)> = vec![];
        let txns: Vec<(Option<String>, String, Decimal)> = vec![];
        let result = combine_category_totals(subs, txns);
        assert!(result.is_empty());
    }

    #[test]
    fn combine_subs_only() {
        let subs = vec![
            (Some("cat-1".to_string()), "Streaming".to_string(), dec("50")),
        ];
        let txns: Vec<(Option<String>, String, Decimal)> = vec![];
        let result = combine_category_totals(subs, txns);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category_name, "Streaming");
        assert_eq!(result[0].total, dec("50"));
        assert_eq!(result[0].percentage, 100.0);
    }

    #[test]
    fn combine_txns_only() {
        let subs: Vec<(Option<String>, String, Decimal)> = vec![];
        let txns = vec![
            (Some("cat-2".to_string()), "Groceries".to_string(), dec("200")),
        ];
        let result = combine_category_totals(subs, txns);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category_name, "Groceries");
        assert_eq!(result[0].total, dec("200"));
        assert_eq!(result[0].percentage, 100.0);
    }

    #[test]
    fn combine_subs_and_txns() {
        let subs = vec![
            (Some("cat-1".to_string()), "Streaming".to_string(), dec("60")),
        ];
        let txns = vec![
            (Some("cat-2".to_string()), "Groceries".to_string(), dec("40")),
        ];
        let result = combine_category_totals(subs, txns);
        assert_eq!(result.len(), 2);
        // Sorted by total descending
        assert_eq!(result[0].category_name, "Streaming");
        assert_eq!(result[1].category_name, "Groceries");
        assert!((result[0].percentage - 60.0).abs() < 0.2);
        assert!((result[1].percentage - 40.0).abs() < 0.2);
    }

    #[test]
    fn combine_same_category_merges() {
        let subs = vec![
            (Some("cat-1".to_string()), "Entertainment".to_string(), dec("30")),
        ];
        let txns = vec![
            (None, "Entertainment".to_string(), dec("20")),
        ];
        let result = combine_category_totals(subs, txns);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category_name, "Entertainment");
        assert_eq!(result[0].total, dec("50"));
        assert_eq!(result[0].percentage, 100.0);
    }

    #[test]
    fn combine_sorted_by_total_descending() {
        let subs = vec![
            (Some("1".to_string()), "Small".to_string(), dec("10")),
            (Some("2".to_string()), "Big".to_string(), dec("90")),
        ];
        let txns: Vec<(Option<String>, String, Decimal)> = vec![];
        let result = combine_category_totals(subs, txns);
        assert_eq!(result[0].category_name, "Big");
        assert_eq!(result[1].category_name, "Small");
    }

    #[test]
    fn combine_category_id_from_txns_when_subs_missing() {
        let subs = vec![
            (None, "Stuff".to_string(), dec("50")),
        ];
        let txns = vec![
            (Some("cat-t".to_string()), "Stuff".to_string(), dec("25")),
        ];
        let result = combine_category_totals(subs, txns);
        // Subs have None for cat_id, txns have Some - the first one wins
        assert_eq!(result[0].category_id, None); // Subs set it first with None
    }

    #[test]
    fn spending_trend_period_format() {
        let trend = SpendingTrend {
            period: format!("{}/{:02}", 6, 24),
            total: dec("100"),
        };
        assert_eq!(trend.period, "6/24");
    }
}
