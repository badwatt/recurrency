use chrono::Datelike;
use rust_decimal::Decimal;
use sqlx::PgPool;
use sqlx::types::Json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::reports::models::{CategoryBreakdown, MonthlySummary, ReportResponse, SpendingTrend};
use crate::shared::date_utils::days_in_month;
use crate::shared::error::AppResult;
use crate::currency::service::convert;
use crate::subscriptions::models::Recurrence;

/// Trend chart shows six previous months, the current month and three future months.
pub const TREND_PAST_MONTHS: i32 = 6;
pub const TREND_FUTURE_MONTHS: i32 = 3;

pub struct ReportService;

/// Report port, injected into `AppState` as `Arc<dyn ReportServiceTrait>`.
#[async_trait::async_trait]
pub trait ReportServiceTrait: Send + Sync {
    async fn get_monthly_summary(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<MonthlySummary>;
    async fn get_category_breakdown(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<Vec<CategoryBreakdown>>;
    async fn get_recent_trends(
        &self,
        user_id: Uuid,
        months_count: i32,
    ) -> AppResult<Vec<SpendingTrend>>;
    async fn get_full_report(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<ReportResponse>;
}

/// SQLx adapter delegating to the existing, tested [`ReportService`] functions.
pub struct SqlxReportService {
    pool: PgPool,
}

impl SqlxReportService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ReportServiceTrait for SqlxReportService {
    async fn get_monthly_summary(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<MonthlySummary> {
        ReportService::get_monthly_summary(&self.pool, user_id, year, month).await
    }

    async fn get_category_breakdown(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<Vec<CategoryBreakdown>> {
        ReportService::get_category_breakdown(&self.pool, user_id, year, month).await
    }

    async fn get_recent_trends(
        &self,
        user_id: Uuid,
        months_count: i32,
    ) -> AppResult<Vec<SpendingTrend>> {
        ReportService::get_recent_trends(&self.pool, user_id, months_count).await
    }

    async fn get_full_report(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<ReportResponse> {
        ReportService::get_full_report(&self.pool, user_id, year, month).await
    }
}

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
        (total / overall_total * Decimal::from(100))
            .round_dp(1)
            .to_string()
            .parse()
            .unwrap_or(0.0)
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

    breakdown.sort_by_key(|b| std::cmp::Reverse(b.total));
    breakdown
}

type SubscriptionRow = (
    Option<String>,
    String,
    Decimal,
    String,
    Json<Recurrence>,
    chrono::NaiveDate,
    Option<chrono::NaiveDate>,
);

impl ReportService {
    pub async fn get_monthly_summary(
        pool: &PgPool,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<MonthlySummary> {
        let (from_date, to_date) = month_date_range(year, month);

        let base: String = sqlx::query_scalar("SELECT currency FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
        let rates = crate::currency::service::get_rates_for_user(pool, user_id, &base).await?;
        let subscriptions = sqlx::query_as::<_, (Decimal, String, Json<Recurrence>, chrono::NaiveDate, Option<chrono::NaiveDate>)>(
            "SELECT amount, currency, recurrence, start_date, end_date FROM subscriptions WHERE user_id = $1 AND is_active = true",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        let mut subscriptions_count = 0i64;
        let mut subscriptions_total = Decimal::ZERO;
        for (amount, currency, recurrence, start, end) in subscriptions {
            let occurrences = crate::subscriptions::recurrence::occurrences_in_month(
                start,
                &recurrence.0,
                end,
                year,
                month,
                amount,
            );
            if !occurrences.is_empty() {
                subscriptions_count += 1;
                subscriptions_total += occurrences
                    .into_iter()
                    .map(|(_, amount)| convert(amount, &currency, &base, &rates))
                    .sum::<Decimal>();
            }
        }
        let _ = (from_date, to_date);

        let transaction_rows: Vec<(Decimal, String)> = sqlx::query_as(
            r#"
            SELECT amount, currency
            FROM transactions
            WHERE user_id = $1 AND date >= $2 AND date <= $3
            "#,
        )
        .bind(user_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(pool)
        .await?;
        let transactions_count = i64::try_from(transaction_rows.len()).unwrap_or(0);
        let transactions_total: Decimal = transaction_rows
            .into_iter()
            .map(|(amount, currency)| convert(amount, &currency, &base, &rates))
            .sum();

        Ok(MonthlySummary {
            year,
            month,
            subscriptions_total,
            transactions_total,
            overall_total: subscriptions_total + transactions_total,
            subscriptions_count,
            transactions_count,
        })
    }

    pub async fn get_category_breakdown(
        pool: &PgPool,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<Vec<CategoryBreakdown>> {
        let (from_date, to_date) = month_date_range(year, month);
        let base: String = sqlx::query_scalar("SELECT currency FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
        let rates = crate::currency::service::get_rates_for_user(pool, user_id, &base).await?;

        // Get subscriptions and filter by the first scheduled charge that
        // falls in this month. A future activation does not count early.
        let subscription_rows: Vec<SubscriptionRow> = sqlx::query_as(
            r#"
            SELECT c.id::text, COALESCE(c.name, 'Uncategorized'), s.amount, s.currency,
                   s.recurrence, s.start_date, s.end_date
            FROM subscriptions s
            LEFT JOIN categories c ON s.category_id = c.id AND c.user_id = s.user_id
            WHERE s.user_id = $1 AND s.is_active = true
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut subs_by_cat_map: HashMap<String, (Option<String>, Decimal)> = HashMap::new();
        for (cat_id, name, amount, currency, recurrence, start, end) in subscription_rows {
            let total: Decimal = crate::subscriptions::recurrence::occurrences_in_month(
                start,
                &recurrence.0,
                end,
                year,
                month,
                amount,
            )
            .into_iter()
            .map(|(_, amount)| convert(amount, &currency, &base, &rates))
            .sum();
            if total.is_zero() {
                continue;
            }
            let entry = subs_by_cat_map.entry(name).or_insert((cat_id, Decimal::ZERO));
            entry.1 += total;
        }
        let _ = (from_date, to_date);
        let subs_by_cat: Vec<(Option<String>, String, Decimal)> = subs_by_cat_map
            .into_iter()
            .map(|(name, (id, total))| (id, name, total))
            .collect();

        // Get transactions by category and convert each row before aggregating.
        let txn_rows: Vec<(Option<String>, String, Decimal, String)> = sqlx::query_as(
            r#"
            SELECT c.id::text, COALESCE(c.name, 'Uncategorized'), t.amount, t.currency
            FROM transactions t
            LEFT JOIN categories c ON t.category_id = c.id AND c.user_id = t.user_id
            WHERE t.user_id = $1 AND t.date >= $2 AND t.date <= $3
            "#,
        )
        .bind(user_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(pool)
        .await?;
        let mut txn_map: HashMap<String, (Option<String>, Decimal)> = HashMap::new();
        for (cat_id, name, amount, currency) in txn_rows {
            let entry = txn_map.entry(name).or_insert((cat_id, Decimal::ZERO));
            entry.1 += convert(amount, &currency, &base, &rates);
        }
        let txns_by_cat: Vec<(Option<String>, String, Decimal)> = txn_map
            .into_iter()
            .map(|(name, (id, total))| (id, name, total))
            .collect();

        // Combine and aggregate using extracted function
        Ok(combine_category_totals(subs_by_cat, txns_by_cat))
    }

    pub async fn get_recent_trends(
        pool: &PgPool,
        user_id: Uuid,
        months_count: i32,
    ) -> AppResult<Vec<SpendingTrend>> {
        let months = trend_months(
            chrono::Utc::now().date_naive(),
            months_count,
            TREND_FUTURE_MONTHS,
        );
        if months.is_empty() {
            return Ok(Vec::new());
        }

        let base: String = sqlx::query_scalar("SELECT currency FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
        let rates = crate::currency::service::get_rates_for_user(pool, user_id, &base).await?;
        let subscriptions = sqlx::query_as::<
            _,
            (Decimal, String, Json<Recurrence>, chrono::NaiveDate, Option<chrono::NaiveDate>),
        >(
            "SELECT amount, currency, recurrence, start_date, end_date FROM subscriptions WHERE user_id = $1 AND is_active = true",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let earliest = months
            .iter()
            .map(|m| m.2)
            .min()
            .expect("months is non-empty");
        let latest = months
            .iter()
            .map(|m| m.3)
            .max()
            .expect("months is non-empty");

        let by_month: Vec<(i32, i32, Decimal, String)> = sqlx::query_as(
            r#"
            SELECT EXTRACT(YEAR FROM date)::int AS year,
                   EXTRACT(MONTH FROM date)::int AS month,
                   amount, currency
            FROM transactions
            WHERE user_id = $1 AND date >= $2 AND date <= $3
            "#,
        )
        .bind(user_id)
        .bind(earliest)
        .bind(latest)
        .fetch_all(pool)
        .await?;

        let mut transaction_totals: HashMap<(i32, u32), Decimal> = HashMap::new();
        for (year, month, amount, currency) in by_month {
            if let Ok(month) = u32::try_from(month) {
                *transaction_totals.entry((year, month)).or_default() +=
                    convert(amount, &currency, &base, &rates);
            }
        }

        Ok(months
            .into_iter()
            .map(|(year, month, month_start, month_end)| {
                let subscription_total = subscriptions
                    .iter()
                    .map(|(amount, currency, recurrence, start, end)| {
                        crate::subscriptions::recurrence::occurrences_in_month(
                            *start,
                            &recurrence.0,
                            *end,
                            year,
                            month,
                            *amount,
                        )
                        .into_iter()
                        .map(|(_, amount)| convert(amount, currency, &base, &rates))
                        .sum::<Decimal>()
                    })
                    .sum::<Decimal>();
                let _ = (month_start, month_end);
                SpendingTrend {
                    period: format!("{}/{:02}", month, year % 100),
                    total: subscription_total
                        + transaction_totals
                            .get(&(year, month))
                            .copied()
                            .unwrap_or(Decimal::ZERO),
                }
            })
            .collect())
    }

    pub async fn get_full_report(
        pool: &PgPool,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<ReportResponse> {
        let monthly_summary = Self::get_monthly_summary(pool, user_id, year, month).await?;
        let category_breakdown = Self::get_category_breakdown(pool, user_id, year, month).await?;
        let recent_trends = Self::get_recent_trends(pool, user_id, TREND_PAST_MONTHS).await?;

        Ok(ReportResponse {
            monthly_summary,
            category_breakdown,
            recent_trends,
        })
    }
}

#[allow(dead_code)]
pub(crate) fn normalize_to_monthly(amount: Decimal, frequency: &str) -> Decimal {
    match frequency {
        "daily" => amount * Decimal::from(30),
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

/// Calendar months covered by a recent-trends window, newest first.
/// `today` is injected so the window is deterministic in tests.
#[allow(dead_code)]
pub(crate) fn recent_months(
    today: chrono::NaiveDate,
    months_count: i32,
) -> Vec<(i32, u32, chrono::NaiveDate, chrono::NaiveDate)> {
    if months_count <= 0 {
        return Vec::new();
    }
    (0..months_count)
        .map(|i| {
            let date = subtract_months_from_date(today, i);
            let (from, to) = month_date_range(date.year(), date.month());
            (date.year(), date.month(), from, to)
        })
        .collect()
}

/// Months for the spending-trend chart: future months, current month, then past
/// months. The result stays newest-first for the API; the UI reverses it.
pub(crate) fn trend_months(
    today: chrono::NaiveDate,
    past_months: i32,
    future_months: i32,
) -> Vec<(i32, u32, chrono::NaiveDate, chrono::NaiveDate)> {
    let mut months = Vec::new();
    for offset in (1..=future_months.max(0)).rev() {
        let date = subtract_months_from_date(today, -offset);
        let (from, to) = month_date_range(date.year(), date.month());
        months.push((date.year(), date.month(), from, to));
    }
    let (from, to) = month_date_range(today.year(), today.month());
    months.push((today.year(), today.month(), from, to));
    for offset in 1..=past_months.max(0) {
        let date = subtract_months_from_date(today, offset);
        let (from, to) = month_date_range(date.year(), date.month());
        months.push((date.year(), date.month(), from, to));
    }
    months
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(
            normalize_to_monthly(Decimal::ZERO, "monthly"),
            Decimal::ZERO
        );
        assert_eq!(
            normalize_to_monthly(Decimal::ZERO, "quarterly"),
            Decimal::ZERO
        );
        assert_eq!(normalize_to_monthly(Decimal::ZERO, "yearly"), Decimal::ZERO);
    }

    // ── subtract_months_from_date ──

    #[test]
    fn subtract_months_simple() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subtract_months_from_date(d, 1);
        assert_eq!(
            result,
            chrono::NaiveDate::from_ymd_opt(2024, 5, 15).unwrap()
        );
    }

    #[test]
    fn subtract_months_cross_year() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        let result = subtract_months_from_date(d, 3);
        assert_eq!(
            result,
            chrono::NaiveDate::from_ymd_opt(2023, 11, 15).unwrap()
        );
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
        assert_eq!(
            result,
            chrono::NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
    }

    #[test]
    fn subtract_months_day_clamping_non_leap() {
        let d = chrono::NaiveDate::from_ymd_opt(2023, 3, 31).unwrap(); // non-leap
        let result = subtract_months_from_date(d, 1);
        assert_eq!(
            result,
            chrono::NaiveDate::from_ymd_opt(2023, 2, 28).unwrap()
        );
    }

    #[test]
    fn subtract_months_many_years() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subtract_months_from_date(d, 24);
        assert_eq!(
            result,
            chrono::NaiveDate::from_ymd_opt(2022, 6, 15).unwrap()
        );
    }

    #[test]
    fn subtract_months_may_31_to_april() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 5, 31).unwrap();
        let result = subtract_months_from_date(d, 1);
        assert_eq!(
            result,
            chrono::NaiveDate::from_ymd_opt(2024, 4, 30).unwrap()
        );
    }

    #[test]
    fn subtract_months_preserves_day_within_bounds() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 8, 15).unwrap();
        let result = subtract_months_from_date(d, 5);
        assert_eq!(
            result,
            chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()
        );
    }

    // ── Category percentage calculation (mirrors get_category_breakdown logic) ──

    #[test]
    fn percentage_calculation_with_two_categories() {
        let cat_a = dec("60");
        let cat_b = dec("40");
        let total = cat_a + cat_b;

        let pct_a = if total > Decimal::ZERO {
            (cat_a / total * Decimal::from(100))
                .round_dp(1)
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };
        let pct_b = if total > Decimal::ZERO {
            (cat_b / total * Decimal::from(100))
                .round_dp(1)
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
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
        let (_from, to) = month_date_range(2023, 2);
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
        let subs = vec![(
            Some("cat-1".to_string()),
            "Streaming".to_string(),
            dec("50"),
        )];
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
        let txns = vec![(
            Some("cat-2".to_string()),
            "Groceries".to_string(),
            dec("200"),
        )];
        let result = combine_category_totals(subs, txns);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category_name, "Groceries");
        assert_eq!(result[0].total, dec("200"));
        assert_eq!(result[0].percentage, 100.0);
    }

    #[test]
    fn combine_subs_and_txns() {
        let subs = vec![(
            Some("cat-1".to_string()),
            "Streaming".to_string(),
            dec("60"),
        )];
        let txns = vec![(
            Some("cat-2".to_string()),
            "Groceries".to_string(),
            dec("40"),
        )];
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
        let subs = vec![(
            Some("cat-1".to_string()),
            "Entertainment".to_string(),
            dec("30"),
        )];
        let txns = vec![(None, "Entertainment".to_string(), dec("20"))];
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
        let subs = vec![(None, "Stuff".to_string(), dec("50"))];
        let txns = vec![(Some("cat-t".to_string()), "Stuff".to_string(), dec("25"))];
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

    // ── normalize_to_monthly ──

    #[test]
    fn normalize_monthly_is_unchanged() {
        let result = normalize_to_monthly(dec("10.00"), "monthly");
        assert_eq!(result, dec("10.00"));
    }

    #[test]
    fn normalize_weekly_to_monthly() {
        // weekly * 4.33 ≈ monthly
        let result = normalize_to_monthly(dec("10.00"), "weekly");
        assert_eq!(result, dec("43.30")); // 10 * 52 / 12
    }

    #[test]
    fn normalize_quarterly_to_monthly() {
        let result = normalize_to_monthly(dec("30.00"), "quarterly");
        assert_eq!(result, dec("10.00")); // 30 / 3
    }

    #[test]
    fn normalize_yearly_to_monthly() {
        let result = normalize_to_monthly(dec("120.00"), "yearly");
        assert_eq!(result, dec("10.00")); // 120 / 12
    }

    #[test]
    fn normalize_unknown_frequency() {
        let result = normalize_to_monthly(dec("50.00"), "biweekly");
        assert_eq!(result, dec("50.00")); // fallback: unchanged
    }

    // ── subtract_months_from_date additional coverage ──

    #[test]
    fn subtract_months_one_month() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subtract_months_from_date(d, 1);
        assert_eq!(
            result,
            chrono::NaiveDate::from_ymd_opt(2024, 5, 15).unwrap()
        );
    }

    #[test]
    fn subtract_months_to_previous_year() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let result = subtract_months_from_date(d, 1);
        assert_eq!(
            result,
            chrono::NaiveDate::from_ymd_opt(2023, 12, 15).unwrap()
        );
    }

    #[test]
    fn subtract_months_multiple_years() {
        let d = chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let result = subtract_months_from_date(d, 24);
        assert_eq!(result, chrono::NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());
    }

    #[test]
    fn subtract_months_zero_unchanged() {
        let d = chrono::NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
        let result = subtract_months_from_date(d, 0);
        assert_eq!(result, d);
    }

    #[test]
    fn subtract_months_jan_to_dec() {
        // Jan 31 - 1 month = Dec 31
        let d = chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let result = subtract_months_from_date(d, 1);
        assert_eq!(
            result,
            chrono::NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()
        );
    }

    // calculate_percentage edge cases
    #[test]
    fn calculate_percentage_zero_previous() {
        let result = calculate_percentage(dec("100.00"), dec("0.00"));
        assert_eq!(result, 0.0);
    }

    #[test]
    fn calculate_percentage_negative_previous() {
        let result = calculate_percentage(dec("100.00"), dec("-50.00"));
        assert_eq!(result, 0.0); // invalid previous
    }

    #[test]
    fn calculate_percentage_100_percent() {
        let result = calculate_percentage(dec("100.00"), dec("100.00"));
        assert_eq!(result, 100.0);
    }

    #[test]
    fn calculate_percentage_200_percent() {
        let result = calculate_percentage(dec("200.00"), dec("100.00"));
        assert_eq!(result, 200.0);
    }

    // combine_category_totals edge cases
    #[test]
    fn combine_category_totals_empty() {
        let existing: Vec<(Option<String>, String, Decimal)> = vec![];
        let incoming: Vec<(Option<String>, String, Decimal)> = vec![];
        let result = combine_category_totals(existing, incoming);
        assert!(result.is_empty());
    }

    #[test]
    fn combine_category_totals_merge_same() {
        let existing = vec![(None, "Food".to_string(), dec("50.00"))];
        let incoming = vec![(None, "Food".to_string(), dec("30.00"))];
        let result = combine_category_totals(existing, incoming);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].total, dec("80.00"));
    }

    #[test]
    fn combine_category_totals_different_categories() {
        let existing = vec![(None, "Food".to_string(), dec("50.00"))];
        let incoming = vec![(
            Some("cat2".to_string()),
            "Transport".to_string(),
            dec("25.00"),
        )];
        let result = combine_category_totals(existing, incoming);
        assert_eq!(result.len(), 2);
    }

    // ReportService struct
    #[test]
    fn report_service_struct() {
        let _ = ReportService;
    }
}

// ── calculate_percentage edge cases ──
#[test]
fn calc_percentage_zero_total() {
    let result = calculate_percentage(Decimal::from(100), Decimal::ZERO);
    assert!(result.is_nan() || result == 0.0);
}

#[test]
fn calc_percentage_zero_overall() {
    let result = calculate_percentage(Decimal::ZERO, Decimal::from(100));
    assert!(result.is_nan() || result == 0.0);
}

#[test]
fn calc_percentage_both_zero() {
    let result = calculate_percentage(Decimal::ZERO, Decimal::ZERO);
    assert!(result.is_nan() || result == 0.0);
}

#[test]
fn calc_percentage_100_percent() {
    let result = calculate_percentage(Decimal::from(50), Decimal::from(50));
    assert!((result - 100.0).abs() < 0.001);
}

#[test]
fn calc_percentage_50_percent() {
    let result = calculate_percentage(Decimal::from(25), Decimal::from(50));
    assert!((result - 50.0).abs() < 0.001);
}

// ── month_date_range tests ──
#[test]
fn month_date_range_january() {
    let (start, end) = month_date_range(2024, 1);
    assert_eq!(start.month(), 1);
    assert_eq!(end.month(), 1);
    assert_eq!(start.day(), 1);
}

#[test]
fn month_date_range_february_leap() {
    let (start, end) = month_date_range(2024, 2);
    assert_eq!(start.month(), 2);
    assert_eq!(end.day(), 29);
}

#[test]
fn month_date_range_february_non_leap() {
    let (start, end) = month_date_range(2023, 2);
    assert_eq!(start.month(), 2);
    assert_eq!(end.day(), 28);
}

#[test]
fn month_date_range_all_months() {
    for m in 1..=12 {
        let (start, end) = month_date_range(2024, m);
        assert_eq!(start.month(), m);
        assert_eq!(end.month(), m);
        assert!(end >= start);
    }
}

// ── normalize_to_monthly edge cases ──
#[test]
fn normalize_weekly_precise() {
    let result = normalize_to_monthly(Decimal::from(52), "weekly");
    assert!(result > Decimal::ZERO);
}

#[test]
fn normalize_monthly_large() {
    let result = normalize_to_monthly(Decimal::from(999999), "monthly");
    assert_eq!(result, Decimal::from(999999));
}

// ── subtract_months edge cases ──
#[test]
fn subtract_months_many() {
    let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let result = subtract_months_from_date(date, 12);
    assert!(result < date);
}

#[test]
fn subtract_months_january_to_december() {
    let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let result = subtract_months_from_date(date, 1);
    assert_eq!(result.month(), 12);
    assert_eq!(result.year(), 2023);
}

#[test]
fn recent_months_newest_first_and_wraps_year() {
    let today = chrono::NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
    let months = recent_months(today, 4);
    let labels: Vec<(i32, u32)> = months.iter().map(|m| (m.0, m.1)).collect();
    assert_eq!(labels, vec![(2024, 2), (2024, 1), (2023, 12), (2023, 11)]);
    assert_eq!(
        months[0].2,
        chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()
    );
    assert_eq!(
        months[0].3,
        chrono::NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
    );
}

#[test]
fn recent_months_non_positive_is_empty() {
    let today = chrono::NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
    assert!(recent_months(today, 0).is_empty());
    assert!(recent_months(today, -3).is_empty());
}

#[test]
fn trend_months_centers_current_month() {
    let today = chrono::NaiveDate::from_ymd_opt(2026, 11, 12).unwrap();
    let months = trend_months(today, 2, 2);
    assert_eq!(months.len(), 5);
    assert_eq!((months[0].0, months[0].1), (2027, 1));
    assert_eq!((months[2].0, months[2].1), (2026, 11));
    assert_eq!((months[4].0, months[4].1), (2026, 9));
}
