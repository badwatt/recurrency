use chrono::NaiveDate;

/// Format a NaiveDate according to user's date format preference.
/// Supports "DD-MM-YYYY" and "YYYY-MM-DD" formats.
pub fn format_date(date: &NaiveDate, format: &str) -> String {
    match format {
        "YYYY-MM-DD" => date.format("%Y-%m-%d").to_string(),
        "DD-MM-YYYY" | _ => date.format("%d-%m-%Y").to_string(),
    }
}

/// Format a NaiveDate as short display: "Mon, Jan 5" style.
pub fn format_display_date(date: &NaiveDate) -> String {
    date.format("%a, %b %-d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_date_dd_mm_yyyy() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(format_date(&date, "DD-MM-YYYY"), "15-06-2024");
    }

    #[test]
    fn format_date_yyyy_mm_dd() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(format_date(&date, "YYYY-MM-DD"), "2024-06-15");
    }

    #[test]
    fn format_date_unknown_format_defaults_to_dd_mm_yyyy() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(format_date(&date, "INVALID"), "15-06-2024");
    }

    #[test]
    fn format_date_default_is_dd_mm_yyyy() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(format_date(&date, ""), "15-06-2024");
    }

    #[test]
    fn format_date_leading_zeros() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        assert_eq!(format_date(&date, "DD-MM-YYYY"), "05-01-2024");
        assert_eq!(format_date(&date, "YYYY-MM-DD"), "2024-01-05");
    }

    #[test]
    fn format_date_with_display_style() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = format_display_date(&date);
        assert!(result.contains("Jun"));
        assert!(result.contains("15"));
    }
}
