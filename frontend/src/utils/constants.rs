/// Supported currencies with display labels.
/// Mirrors the CURRENCIES constant from the React frontend.
pub const CURRENCIES: &[(&str, &str)] = &[
    ("EUR", "EUR (€)"),
    ("USD", "USD ($)"),
    ("GBP", "GBP (£)"),
    ("JPY", "JPY (¥)"),
    ("CAD", "CAD ($)"),
    ("AUD", "AUD ($)"),
];

/// Supported billing frequencies.
pub const FREQUENCIES: &[(&str, &str)] = &[
    ("daily", "Daily"),
    ("weekly", "Weekly"),
    ("monthly", "Monthly"),
    ("quarterly", "Quarterly"),
    ("yearly", "Yearly"),
];


/// Recurrence units for filters and summaries.
pub const RECURRENCE_UNITS: &[(&str, &str)] = &[
    ("day", "Days"),
    ("week", "Weeks"),
    ("month", "Months"),
    ("year", "Years"),
];

/// Default currency.
pub const DEFAULT_CURRENCY: &str = "EUR";

/// App version (workspace Cargo.toml is the single source).
#[allow(dead_code)]
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currencies_has_eur_first() {
        assert_eq!(CURRENCIES[0].0, "EUR");
    }

    #[test]
    fn currencies_count() {
        assert_eq!(CURRENCIES.len(), 6);
    }

    #[test]
    fn currencies_contains_all_expected() {
        let codes: Vec<&str> = CURRENCIES.iter().map(|(k, _)| *k).collect();
        assert!(codes.contains(&"EUR"));
        assert!(codes.contains(&"USD"));
        assert!(codes.contains(&"GBP"));
        assert!(codes.contains(&"JPY"));
        assert!(codes.contains(&"CAD"));
        assert!(codes.contains(&"AUD"));
    }

    #[test]
    fn currencies_labels_match_codes() {
        for (code, label) in CURRENCIES {
            assert!(label.contains(code));
        }
    }

    #[test]
    fn frequencies_count() {
        assert_eq!(FREQUENCIES.len(), 5);
    }

    #[test]
    fn frequencies_contains_all_expected() {
        let freqs: Vec<&str> = FREQUENCIES.iter().map(|(k, _)| *k).collect();
        assert!(freqs.contains(&"daily"));
        assert!(freqs.contains(&"weekly"));
        assert!(freqs.contains(&"monthly"));
        assert!(freqs.contains(&"quarterly"));
        assert!(freqs.contains(&"yearly"));
    }

    #[test]
    fn frequencies_labels_capitalized() {
        for (_, label) in FREQUENCIES {
            assert!(label.chars().next().unwrap().is_uppercase());
        }
    }

    #[test]
    fn default_currency_is_eur() {
        assert_eq!(DEFAULT_CURRENCY, "EUR");
    }

    #[test]
    fn currency_labels() {
        assert_eq!(CURRENCIES[0].1, "EUR (€)");
        assert_eq!(CURRENCIES[1].1, "USD ($)");
    }

    #[test]
    fn app_version_is_set() {
        assert!(!APP_VERSION.is_empty());
    }
}
