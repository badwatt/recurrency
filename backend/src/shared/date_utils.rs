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
}