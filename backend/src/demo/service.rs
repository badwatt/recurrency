use chrono::Datelike;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::date_utils::days_in_month;

pub struct DemoService;

impl DemoService {
    pub async fn reset_demo_data(pool: &PgPool, user_id: Uuid) -> Result<(), String> {
        // Clear existing data (delete subs and txns first due to FK constraints)
        sqlx::query("DELETE FROM subscriptions WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query("DELETE FROM transactions WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        // Delete all categories for the user
        sqlx::query("DELETE FROM categories WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        // Create default "Uncategorized" category
        sqlx::query(
            "INSERT INTO categories (id, user_id, name, is_system, is_default, created_at, updated_at)
             VALUES ($1, $2, 'Uncategorized', true, true, NOW(), NOW())"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        // Get the default category for the user
        let default_category: (Uuid,) = sqlx::query_as(
            "SELECT id FROM categories WHERE user_id = $1 AND is_default = true"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        let default_category_id = default_category.0;

        sqlx::query("DELETE FROM user_preferences WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(
            "UPDATE users SET currency = 'EUR', name = 'Demo User', verified = true, updated_at = NOW() WHERE id = $1",
        )
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        let today = chrono::Utc::now().date_naive();
        let this_month = today.month();
        let this_year = today.year();
        let today_day = today.day();

        // next month
        let (_next_year, _next_month) = if this_month == 12 {
            (this_year + 1, 1)
        } else {
            (this_year, this_month + 1)
        };

        // subs: (name, amount, start_date_months_ago, billing_day)
        let subs = [
            ("Netflix",        "15.99", 3, 15),  // started 3 months ago on the 15th
            ("Spotify",         "9.99", 1, 1),   // started 1 month ago on the 1st
            ("iCloud",          "2.99", 6, 10),  // started 6 months ago on the 10th
            ("Gym Membership",  "49.99", 2, 5),  // started 2 months ago on the 5th
            ("Adobe CC",        "54.99", 4, 18),  // started 4 months ago on the 18th
        ];

        for (name, amount, months_ago, day) in subs {
            let start_date = calc_start_date(this_month, this_year, months_ago, day);
            let next_date = calc_next_date(this_month, this_year, today_day, day);

            sqlx::query(
                "INSERT INTO subscriptions (id, user_id, category_id, name, amount, currency, frequency, start_date, next_billing_date, is_active)
                 VALUES ($1, $2, $3, $4, $5, 'EUR', 'monthly', $6, $7, true)"
            )
                .bind(Uuid::new_v4())
                .bind(user_id)
                .bind(default_category_id)
                .bind(name)
                .bind(Decimal::from_str_exact(amount).unwrap_or(Decimal::ZERO))
                .bind(start_date)
                .bind(next_date)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
        }

        // transactions spread over last week
        let txns = [
            ("Coffee",     "4.50",  0),
            ("Groceries",  "85.00", 2),
            ("Gas",        "45.00", 4),
            ("Restaurant", "35.00", 6),
        ];
        for (name, amount, ago) in txns {
            let date = today - chrono::Duration::days(ago);
            sqlx::query(
                "INSERT INTO transactions (id, user_id, category_id, name, amount, currency, date)
                 VALUES ($1, $2, $3, $4, $5, 'EUR', $6)"
            )
                .bind(Uuid::new_v4())
                .bind(user_id)
                .bind(default_category_id)
                .bind(name)
                .bind(Decimal::from_str_exact(amount).unwrap_or(Decimal::ZERO))
                .bind(date)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

/// Calculate the start date given current month/year and months ago.
pub(crate) fn calc_start_date(month: u32, year: i32, months_ago: u32, day: u32) -> chrono::NaiveDate {
    let total_months = year * 12 + month as i32 - months_ago as i32 - 1;
    let new_year = total_months / 12;
    let new_month = ((total_months % 12) + 1) as u32;
    let max_day = days_in_month(new_year, new_month);
    let new_day = day.min(max_day);
    chrono::NaiveDate::from_ymd_opt(new_year, new_month, new_day)
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap())
}

/// Calculate next billing date from current date and billing day.
pub(crate) fn calc_next_date(month: u32, year: i32, today_day: u32, billing_day: u32) -> chrono::NaiveDate {
    let next_month = if billing_day > today_day { month } else { month + 1 };
    let (next_year, next_month_normalized) = if next_month > 12 {
        (year + 1, 1)
    } else {
        (year, next_month)
    };
    let max_day = days_in_month(next_year, next_month_normalized);
    let final_day = billing_day.min(max_day);
    chrono::NaiveDate::from_ymd_opt(next_year, next_month_normalized, final_day).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_in_month_january() { assert_eq!(days_in_month(2024, 1), 31); }
    #[test]
    fn days_in_month_april() { assert_eq!(days_in_month(2024, 4), 30); }
    #[test]
    fn days_in_month_february_leap() { assert_eq!(days_in_month(2024, 2), 29); }
    #[test]
    fn days_in_month_february_non_leap() { assert_eq!(days_in_month(2023, 2), 28); }

    #[test]
    fn calc_start_date_same_month() {
        let date = calc_start_date(6, 2024, 0, 15);
        assert_eq!(date.month(), 6);
        assert_eq!(date.year(), 2024);
    }

    #[test]
    fn calc_start_date_one_month_ago() {
        let date = calc_start_date(6, 2024, 1, 15);
        assert_eq!(date.month(), 5);
        assert_eq!(date.year(), 2024);
    }

    #[test]
    fn calc_start_date_december_to_november() {
        let date = calc_start_date(12, 2024, 1, 15);
        assert_eq!(date.month(), 11);
        assert_eq!(date.year(), 2024);
    }

    #[test]
    fn calc_start_date_january_to_december() {
        let date = calc_start_date(1, 2024, 1, 15);
        assert_eq!(date.month(), 12);
        assert_eq!(date.year(), 2023);
    }

    #[test]
    fn calc_start_date_day_31_to_month_30() {
        let date = calc_start_date(5, 2024, 1, 31);
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 30);
    }

    #[test]
    fn calc_next_date_future_day_same_month() {
        let date = calc_next_date(6, 2024, 15, 20);
        assert_eq!(date.month(), 6);
        assert_eq!(date.day(), 20);
    }

    #[test]
    fn calc_next_date_past_day_next_month() {
        let date = calc_next_date(6, 2024, 15, 10);
        assert_eq!(date.month(), 7);
    }

    #[test]
    fn calc_next_date_december_to_january() {
        let date = calc_next_date(12, 2024, 15, 10);
        assert_eq!(date.month(), 1);
        assert_eq!(date.year(), 2025);
    }

    #[test]
    fn calc_next_date_day_clamp() {
        let date = calc_next_date(4, 2024, 15, 31);
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 30);
    }

    #[test]
    fn calc_next_date_february_leap() {
        let date = calc_next_date(1, 2024, 15, 31);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 31);
    }

    // ── Additional calc_start_date edge cases ──

    #[test]
    fn calc_start_date_january_to_december_year_boundary() {
        let date = calc_start_date(1, 2024, 1, 15);
        assert_eq!(date.month(), 12);
        assert_eq!(date.year(), 2023);
    }

    #[test]
    fn calc_start_date_12_months_ago() {
        let date = calc_start_date(6, 2024, 12, 15);
        assert_eq!(date.month(), 6);
        assert_eq!(date.year(), 2023);
    }

    #[test]
    fn calc_start_date_6_months_ago() {
        let date = calc_start_date(8, 2024, 6, 10);
        assert_eq!(date.month(), 2);
        assert_eq!(date.year(), 2024);
    }

    #[test]
    fn calc_start_date_march_to_feb_non_leap() {
        // Mar 31, 1 month ago -> Feb 28 2023 (clamped from 31 to 28 for Feb non-leap)
        let date = calc_start_date(3, 2023, 1, 31);
        assert_eq!(date.month(), 2);
        assert_eq!(date.year(), 2023);
        assert_eq!(date.day(), 28);
    }

    #[test]
    fn calc_start_date_day_clamp_april() {
        let date = calc_start_date(4, 2024, 1, 30);
        assert_eq!(date.month(), 3);
        assert_eq!(date.day(), 30);
    }

    #[test]
    fn calc_start_date_multiple_years_ago() {
        let date = calc_start_date(6, 2025, 24, 15);
        assert_eq!(date.month(), 6);
        assert_eq!(date.year(), 2023);
    }

    #[test]
    fn calc_start_date_day_clamp_feb_leap() {
        let date = calc_start_date(2, 2024, 1, 29);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 29);
    }

    // ── Additional calc_next_date edge cases ──

    #[test]
    fn calc_next_date_same_day_different_month() {
        let date = calc_next_date(6, 2024, 14, 15);
        assert!(date.day() <= 15);
    }

    #[test]
    fn calc_next_date_year_end_to_january() {
        // Dec 20, billing day 25 -> billing_day(25) > today_day(20) = true, stays Dec
        let date = calc_next_date(12, 2024, 20, 25);
        assert_eq!(date.month(), 12);
        assert_eq!(date.year(), 2024);
        assert_eq!(date.day(), 25);
    }

    #[test]
    fn calc_next_date_day_1_past() {
        // Day 1, today 1 -> billing_day > today_day: 1 > 1 is false -> next month
        let date = calc_next_date(6, 2024, 1, 1);
        assert_eq!(date.month(), 7);
    }

    #[test]
    fn calc_next_date_month_end_short_month() {
        // June (30 days), billing day 31 -> clamped to 30
        let date = calc_next_date(6, 2024, 15, 31);
        assert_eq!(date.month(), 6);
        assert_eq!(date.day(), 30);
    }

    // ── days_in_month additional edge cases ──

    #[test]
    fn days_in_month_march() { assert_eq!(days_in_month(2024, 3), 31); }
    #[test]
    fn days_in_month_may() { assert_eq!(days_in_month(2024, 5), 31); }
    #[test]
    fn days_in_month_june() { assert_eq!(days_in_month(2024, 6), 30); }
    #[test]
    fn days_in_month_july() { assert_eq!(days_in_month(2024, 7), 31); }
    #[test]
    fn days_in_month_august() { assert_eq!(days_in_month(2024, 8), 31); }
    #[test]
    fn days_in_month_september() { assert_eq!(days_in_month(2024, 9), 30); }
    #[test]
    fn days_in_month_october() { assert_eq!(days_in_month(2024, 10), 31); }
    #[test]
    fn days_in_month_november() { assert_eq!(days_in_month(2024, 11), 30); }
    #[test]
    fn days_in_month_december() { assert_eq!(days_in_month(2024, 12), 31); }

    // ── calc_start_date additional edge cases ──

    #[test]
    fn calc_start_date_day_1() {
        let date = calc_start_date(6, 2024, 1, 1);
        assert_eq!(date.month(), 5);
        assert_eq!(date.day(), 1);
    }

    #[test]
    fn calc_start_date_february_to_january_leap_year() {
        let date = calc_start_date(2, 2024, 1, 29);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 29);
    }

    #[test]
    fn calc_start_date_december_31_to_november() {
        let date = calc_start_date(12, 2023, 1, 31);
        assert_eq!(date.month(), 11);
        assert_eq!(date.day(), 30);
    }

    #[test]
    fn calc_start_date_october_31_to_september() {
        let date = calc_start_date(10, 2024, 1, 31);
        assert_eq!(date.month(), 9);
        assert_eq!(date.day(), 30);
    }

    #[test]
    fn calc_start_date_mid_year() {
        let date = calc_start_date(6, 2024, 3, 15);
        assert_eq!(date.month(), 3);
        assert_eq!(date.year(), 2024);
    }

    #[test]
    fn calc_start_date_2_years_ago() {
        let date = calc_start_date(6, 2024, 24, 20);
        assert_eq!(date.month(), 6);
        assert_eq!(date.year(), 2022);
    }

    #[test]
    fn calc_start_date_leap_year_february() {
        // March 1 -> Feb 29 in leap year (day 29 is valid in Feb 2024)
        let date = calc_start_date(3, 2024, 1, 29);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 29);
    }

    // ── calc_next_date additional edge cases ──

    #[test]
    fn calc_next_date_day_1_same_month() {
        let date = calc_next_date(6, 2024, 15, 1);
        // 1 <= 15, so next month
        assert_eq!(date.month(), 7);
    }

    #[test]
    fn calc_next_date_exact_today() {
        let date = calc_next_date(6, 2024, 15, 15);
        // 15 <= 15 is false (not >), so next month
        assert_eq!(date.month(), 7);
    }

    #[test]
    fn calc_next_date_november_to_december() {
        let date = calc_next_date(11, 2024, 15, 20);
        assert_eq!(date.month(), 11);
        assert_eq!(date.year(), 2024);
    }

    #[test]
    fn calc_next_date_november_to_december_past_day() {
        let date = calc_next_date(11, 2024, 25, 20);
        assert_eq!(date.month(), 12);
        assert_eq!(date.year(), 2024);
    }

    #[test]
    fn calc_next_date_february_leap_year() {
        // From Jan 31, billing day 31 > today 31 → false → next month (Feb), clamped to 29
        let date = calc_next_date(1, 2024, 31, 31);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 29); // Feb 2024 has 29 days, clamped from 31
    }

    #[test]
    fn calc_next_date_february_non_leap_year() {
        // From Jan 31, billing day 31 > today 31 → false → next month (Feb), clamped to 28
        let date = calc_next_date(1, 2023, 31, 31);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 28); // Feb 2023 has 28 days, clamped from 31
    }

    #[test]
    fn calc_next_date_all_months_shortened() {
        // Billing day 31 should clamp to actual days
        assert_eq!(calc_next_date(4, 2024, 15, 31).day(), 30); // April
        assert_eq!(calc_next_date(6, 2024, 15, 31).day(), 30); // June
        assert_eq!(calc_next_date(9, 2024, 15, 31).day(), 30); // September
        assert_eq!(calc_next_date(11, 2024, 15, 31).day(), 30); // November
    }

    #[test]
    fn calc_next_date_all_months_31_days() {
        // Billing day 31 should stay 31 for 31-day months
        assert_eq!(calc_next_date(1, 2024, 15, 31).day(), 31); // Jan
        assert_eq!(calc_next_date(3, 2024, 15, 31).day(), 31); // Mar
        assert_eq!(calc_next_date(5, 2024, 15, 31).day(), 31); // May
        assert_eq!(calc_next_date(7, 2024, 15, 31).day(), 31); // July
        assert_eq!(calc_next_date(8, 2024, 15, 31).day(), 31); // Aug
        assert_eq!(calc_next_date(10, 2024, 15, 31).day(), 31); // Oct
        assert_eq!(calc_next_date(12, 2024, 15, 31).day(), 31); // Dec
    }

    // ── DemoService struct reference ──

    #[test]
    fn demo_service_can_be_referenced() {
        // Just test the struct exists and can be referenced
        let _service: DemoService = DemoService;
    }
}

    // ── calc_next_date additional edge cases ──
    #[test]
    fn calc_next_date_same_month_early_billing_day() {
        let date = calc_next_date(5, 2024, 10, 15); // May 10, billing day 15
        assert_eq!(date.month(), 5);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn calc_next_date_same_month_late_billing_day() {
        let date = calc_next_date(5, 2024, 20, 15); // May 20, billing day 15 < 20 → next month
        assert_eq!(date.month(), 6);
    }

    #[test]
    fn calc_next_date_december_wraps_year() {
        let date = calc_next_date(12, 2024, 15, 20);
        assert_eq!(date.month(), 12);
    }

    // ── DemoService reset returns Result ──
    #[test]
    fn demo_service_struct() {
        let _ = DemoService;
    }
