pub fn days_in_month(year: i32, month: u32) -> u32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn january_has_31() {
        assert_eq!(days_in_month(2024, 1), 31);
    }

    #[test]
    fn april_has_30() {
        assert_eq!(days_in_month(2024, 4), 30);
    }

    #[test]
    fn december_has_31() {
        assert_eq!(days_in_month(2024, 12), 31);
    }

    #[test]
    fn february_leap_year() {
        assert_eq!(days_in_month(2024, 2), 29);
    }

    #[test]
    fn february_non_leap_year() {
        assert_eq!(days_in_month(2023, 2), 28);
    }

    #[test]
    fn february_century_non_leap() {
        assert_eq!(days_in_month(1900, 2), 28);
    }

    #[test]
    fn february_400_year_leap() {
        assert_eq!(days_in_month(2000, 2), 29);
    }

    #[test]
    fn invalid_month_falls_back() {
        // Invalid months return 30 (default branch)
        assert_eq!(days_in_month(2024, 0), 30);
        assert_eq!(days_in_month(2024, 13), 30);
    }

    #[test]
    fn all_31_day_months() {
        assert_eq!(days_in_month(2024, 1), 31);  // Jan
        assert_eq!(days_in_month(2024, 3), 31);  // Mar
        assert_eq!(days_in_month(2024, 5), 31);  // May
        assert_eq!(days_in_month(2024, 7), 31);  // Jul
        assert_eq!(days_in_month(2024, 8), 31);  // Aug
        assert_eq!(days_in_month(2024, 10), 31); // Oct
        assert_eq!(days_in_month(2024, 12), 31); // Dec
    }

    #[test]
    fn all_30_day_months() {
        assert_eq!(days_in_month(2024, 4), 30);  // Apr
        assert_eq!(days_in_month(2024, 6), 30);  // Jun
        assert_eq!(days_in_month(2024, 9), 30);  // Sep
        assert_eq!(days_in_month(2024, 11), 30); // Nov
    }

    #[test]
    fn century_non_leap_year() {
        // 2100 is not a leap year (divisible by 100 but not 400)
        assert_eq!(days_in_month(2100, 2), 28);
    }
}