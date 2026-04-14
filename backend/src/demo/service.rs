use chrono::Datelike;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

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
        let (next_year, next_month) = if this_month == 12 {
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
            let start_date = {
                let (sy, sm) = if this_month as i32 - months_ago <= 0 {
                    (this_year - 1, 12 + this_month as i32 - months_ago)
                } else {
                    (this_year, this_month as i32 - months_ago)
                };
                let sd = day.min(days_in(sy, sm as u32));
                chrono::NaiveDate::from_ymd_opt(sy, sm as u32, sd).unwrap_or(today)
            };
            let next_date = chrono::NaiveDate::from_ymd_opt(
                if day <= today_day { next_year } else { this_year },
                if day <= today_day { next_month } else { this_month },
                day.min(days_in(
                    if day <= today_day { next_year } else { this_year },
                    if day <= today_day { next_month } else { this_month },
                ))
            ).unwrap_or(today);

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

fn days_in(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 },
        _ => 30,
    }
}