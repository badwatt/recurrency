use chrono::NaiveDate;

/// Get system/browser locale string (e.g. "en-US").
#[cfg(target_arch = "wasm32")]
pub fn get_locale() -> String {
    web_sys::window()
        .and_then(|w| w.navigator().language())
        .unwrap_or_else(|| "en-US".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_locale() -> String {
    std::env::var("LANG")
        .ok()
        .and_then(|l| l.split('.').next().map(|s| s.replace('_', "-")))
        .unwrap_or_else(|| "en-US".to_string())
}

/// Map locale to date format pattern.
pub fn get_date_format_from_locale(locale: &str) -> &'static str {
    let lower = locale.to_lowercase();
    if lower.starts_with("en-us") {
        "MM/DD/YYYY"
    } else if lower.starts_with("zh")
        || lower.starts_with("ko")
        || lower.starts_with("ja")
        || lower.starts_with("hu")
        || lower.starts_with("lt")
        || lower.starts_with("mn")
        || lower.starts_with("sv")
    {
        "YYYY/MM/DD"
    } else {
        "DD/MM/YYYY"
    }
}

/// Format a NaiveDate according to format pattern.
/// Supports "DD/MM/YYYY", "YYYY/MM/DD", and "MM/DD/YYYY".
pub fn format_date(date: &NaiveDate, format: &str) -> String {
    match format {
        "YYYY/MM/DD" => date.format("%Y/%m/%d").to_string(),
        "MM/DD/YYYY" => date.format("%m/%d/%Y").to_string(),
        "DD/MM/YYYY" => date.format("%d/%m/%Y").to_string(),
        _ => date.format("%d/%m/%Y").to_string(),
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
        assert_eq!(format_date(&date, "DD/MM/YYYY"), "15/06/2024");
    }

    #[test]
    fn format_date_yyyy_mm_dd() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(format_date(&date, "YYYY/MM/DD"), "2024/06/15");
    }

    #[test]
    fn format_date_mm_dd_yyyy() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(format_date(&date, "MM/DD/YYYY"), "06/15/2024");
    }

    #[test]
    fn format_date_unknown_format_defaults_to_dd_mm_yyyy() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(format_date(&date, "INVALID"), "15/06/2024");
    }

    #[test]
    fn format_date_default_is_dd_mm_yyyy() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(format_date(&date, ""), "15/06/2024");
    }

    #[test]
    fn format_date_leading_zeros() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        assert_eq!(format_date(&date, "DD/MM/YYYY"), "05/01/2024");
        assert_eq!(format_date(&date, "YYYY/MM/DD"), "2024/01/05");
        assert_eq!(format_date(&date, "MM/DD/YYYY"), "01/05/2024");
    }

    #[test]
    fn format_date_with_display_style() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = format_display_date(&date);
        assert!(result.contains("Jun"));
        assert!(result.contains("15"));
    }

    #[test]
    fn locale_en_us_returns_mm_dd_yyyy() {
        assert_eq!(get_date_format_from_locale("en-US"), "MM/DD/YYYY");
        assert_eq!(get_date_format_from_locale("en-us"), "MM/DD/YYYY");
    }

    #[test]
    fn locale_european_returns_dd_mm_yyyy() {
        assert_eq!(get_date_format_from_locale("en-GB"), "DD/MM/YYYY");
        assert_eq!(get_date_format_from_locale("fr-FR"), "DD/MM/YYYY");
        assert_eq!(get_date_format_from_locale("de-DE"), "DD/MM/YYYY");
        assert_eq!(get_date_format_from_locale("es-ES"), "DD/MM/YYYY");
    }

    #[test]
    fn locale_asian_returns_yyyy_mm_dd() {
        assert_eq!(get_date_format_from_locale("ja-JP"), "YYYY/MM/DD");
        assert_eq!(get_date_format_from_locale("ko-KR"), "YYYY/MM/DD");
        assert_eq!(get_date_format_from_locale("zh-CN"), "YYYY/MM/DD");
        assert_eq!(get_date_format_from_locale("hu-HU"), "YYYY/MM/DD");
    }

    #[test]
    fn get_locale_returns_non_empty() {
        let locale = get_locale();
        assert!(!locale.is_empty());
    }
}
