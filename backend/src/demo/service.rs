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
            "INSERT INTO categories (id, user_id, name, is_system, is_default, created_at, updated_at)\n             VALUES ($1, $2, 'Uncategorized', true, true, NOW(), NOW())"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query("DELETE FROM user_preferences WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(
            "UPDATE users SET currency = 'USD', name = 'Demo User', verified = true, updated_at = NOW() WHERE id = $1",
        )
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        let today = chrono::Utc::now().date_naive();
        let this_month = today.month();
        let this_year = today.year();
        let today_day = today.day();
        let days_this = days_in(this_year, this_month);

        // next month
        let (next_year, next_month) = if this_month == 12 {
            (this_year + 1, 1)
        } else {
            (this_year, this_month + 1)
        };
        let days_next = days_in(next_year, next_month);

        // subs: (name, amount, month, day) — always relative to today
        let subs = [
            ("Netflix",        "15.99", this_month, today_day.min(days_this)),
            ("Spotify",         "9.99", this_month, (today_day + 3).min(days_this)),
            ("iCloud",          "2.99", this_month, (today_day + 7).min(days_this)),
            ("Gym Membership",  "49.99", next_month,  5.min(days_next)),
            ("Adobe CC",        "54.99", next_month, 18.min(days_next)),
        ];

        let (ny, nm) = (next_year, next_month);
        for (name, amount, m, d) in subs {
            let yr = if (m == nm && m != this_month) || m > 12 { ny } else { this_year };
            let mo = if m > 12 { 1 } else { m };
            let next_date = chrono::NaiveDate::from_ymd_opt(yr, mo, d).unwrap_or(today);

            sqlx::query(
                "INSERT INTO subscriptions (id, user_id, name, amount, currency, frequency, start_date, next_billing_date, is_active)
                 VALUES ($1, $2, $3, $4, 'USD', 'monthly', $5, $6, true)")
                .bind(Uuid::new_v4())
                .bind(user_id)
                .bind(name)
                .bind(Decimal::from_str_exact(amount).unwrap_or(Decimal::ZERO))
                .bind(today)
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
                "INSERT INTO transactions (id, user_id, name, amount, currency, date)
                 VALUES ($1, $2, $3, $4, 'USD', $5)")
                .bind(Uuid::new_v4())
                .bind(user_id)
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
