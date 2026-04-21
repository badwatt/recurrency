#[cfg(test)]
mod tests {
    use chrono::{Datelike, NaiveDate};
    use rust_decimal::Decimal;

    // Helper to calculate next billing date (mirrors backend logic)
    fn calculate_next_billing_date(current_date: NaiveDate, frequency: &str) -> NaiveDate {
        match frequency {
            "weekly" => current_date + chrono::Duration::days(7),
            "monthly" => {
                let year = current_date.year();
                let month = current_date.month();
                let day = current_date.day();
                let (new_year, new_month) = if month == 12 {
                    (year + 1, 1)
                } else {
                    (year, month + 1)
                };
                let max_day = days_in_month(new_year, new_month);
                NaiveDate::from_ymd_opt(new_year, new_month, day.min(max_day))
                    .unwrap_or(current_date)
            }
            "quarterly" => {
                let year = current_date.year();
                let month = current_date.month();
                let day = current_date.day();
                let new_month = month + 3;
                let (new_year, final_month) = if new_month > 12 {
                    (year + 1, new_month - 12)
                } else {
                    (year, new_month)
                };
                let max_day = days_in_month(new_year, final_month);
                NaiveDate::from_ymd_opt(new_year, final_month, day.min(max_day))
                    .unwrap_or(current_date)
            }
            "yearly" => {
                let year = current_date.year();
                let month = current_date.month();
                let day = current_date.day();
                let new_year = year + 1;
                let max_day = days_in_month(new_year, month);
                NaiveDate::from_ymd_opt(new_year, month, day.min(max_day)).unwrap_or(current_date)
            }
            _ => current_date,
        }
    }

    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    // Normalize amount to monthly (mirrors backend logic)
    fn normalize_to_monthly(amount: Decimal, frequency: &str) -> Decimal {
        match frequency {
            "weekly" => amount * Decimal::from_str_exact("4.33").unwrap(),
            "monthly" => amount,
            "quarterly" => amount / Decimal::from(3),
            "yearly" => amount / Decimal::from(12),
            _ => amount,
        }
    }

    #[test]
    fn test_weekly_billing_date() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let next = calculate_next_billing_date(start, "weekly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 1, 22).unwrap());
    }

    #[test]
    fn test_monthly_billing_date() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let next = calculate_next_billing_date(start, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 2, 15).unwrap());
    }

    #[test]
    fn test_monthly_billing_date_year_boundary() {
        let start = NaiveDate::from_ymd_opt(2026, 12, 15).unwrap();
        let next = calculate_next_billing_date(start, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2027, 1, 15).unwrap());
    }

    #[test]
    fn test_monthly_billing_date_month_end() {
        // Jan 31 -> Feb 28 (non-leap year)
        let start = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let next = calculate_next_billing_date(start, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }

    #[test]
    fn test_monthly_billing_date_leap_year() {
        // Jan 31 -> Feb 29 (leap year)
        let start = NaiveDate::from_ymd_opt(2028, 1, 31).unwrap();
        let next = calculate_next_billing_date(start, "monthly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2028, 2, 29).unwrap());
    }

    #[test]
    fn test_quarterly_billing_date() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let next = calculate_next_billing_date(start, "quarterly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 4, 15).unwrap());
    }

    #[test]
    fn test_quarterly_billing_date_year_boundary() {
        let start = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
        let next = calculate_next_billing_date(start, "quarterly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2027, 2, 15).unwrap());
    }

    #[test]
    fn test_yearly_billing_date() {
        let start = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let next = calculate_next_billing_date(start, "yearly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2027, 6, 15).unwrap());
    }

    #[test]
    fn test_yearly_billing_date_feb_29() {
        // Feb 29, 2028 -> Feb 28, 2029 (non-leap year)
        let start = NaiveDate::from_ymd_opt(2028, 2, 29).unwrap();
        let next = calculate_next_billing_date(start, "yearly");
        assert_eq!(next, NaiveDate::from_ymd_opt(2029, 2, 28).unwrap());
    }

    #[test]
    fn test_normalize_weekly_to_monthly() {
        let weekly = Decimal::from(10);
        let monthly = normalize_to_monthly(weekly, "weekly");
        assert_eq!(monthly, Decimal::from_str_exact("43.30").unwrap());
    }

    #[test]
    fn test_normalize_monthly_stays_same() {
        let monthly = Decimal::from(100);
        let result = normalize_to_monthly(monthly, "monthly");
        assert_eq!(result, Decimal::from(100));
    }

    #[test]
    fn test_normalize_quarterly_to_monthly() {
        let quarterly = Decimal::from(30);
        let monthly = normalize_to_monthly(quarterly, "quarterly");
        assert_eq!(monthly, Decimal::from(10));
    }

    #[test]
    fn test_normalize_yearly_to_monthly() {
        let yearly = Decimal::from(120);
        let monthly = normalize_to_monthly(yearly, "yearly");
        assert_eq!(monthly, Decimal::from(10));
    }

    #[test]
    fn test_days_in_month_january() {
        assert_eq!(days_in_month(2026, 1), 31);
    }

    #[test]
    fn test_days_in_month_february_non_leap() {
        assert_eq!(days_in_month(2026, 2), 28);
    }

    #[test]
    fn test_days_in_month_february_leap() {
        assert_eq!(days_in_month(2028, 2), 29);
    }

    #[test]
    fn test_days_in_month_april() {
        assert_eq!(days_in_month(2026, 4), 30);
    }
}
