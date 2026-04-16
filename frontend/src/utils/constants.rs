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
    ("weekly", "Weekly"),
    ("monthly", "Monthly"),
    ("quarterly", "Quarterly"),
    ("yearly", "Yearly"),
];

/// Default currency.
pub const DEFAULT_CURRENCY: &str = "EUR";

/// App version (read from VERSION file at build time).
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
    fn frequencies_count() {
        assert_eq!(FREQUENCIES.len(), 4);
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
}