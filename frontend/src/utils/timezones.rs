/// Every IANA timezone bundled by `chrono-tz`, sorted for a stable picker.
pub fn all_timezones() -> Vec<&'static str> {
    let mut zones: Vec<&'static str> = chrono_tz::TZ_VARIANTS
        .iter()
        .map(|timezone| timezone.name())
        .collect();
    zones.sort_unstable();
    zones
}

/// Case-insensitive timezone search. Empty input returns the full catalogue.
pub fn filter_timezones(query: &str) -> Vec<&'static str> {
    let query = query.trim().to_lowercase();
    all_timezones()
        .into_iter()
        .filter(|timezone| query.is_empty() || timezone.to_lowercase().contains(&query))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_iana_timezone_is_available() {
        let zones = all_timezones();
        assert!(zones.contains(&"UTC"));
        assert!(zones.contains(&"Europe/Madrid"));
        assert!(zones.contains(&"America/New_York"));
        assert!(zones.len() > 300);
    }

    #[test]
    fn timezone_search_is_case_insensitive_and_partial() {
        assert!(filter_timezones("madrid").contains(&"Europe/Madrid"));
        assert!(filter_timezones("EUROPE/MADRID").contains(&"Europe/Madrid"));
        assert!(filter_timezones("america/").contains(&"America/New_York"));
    }

    #[test]
    fn empty_timezone_search_returns_the_catalogue() {
        assert_eq!(filter_timezones("").len(), all_timezones().len());
        assert_eq!(filter_timezones("   ").len(), all_timezones().len());
    }

    #[test]
    fn unknown_timezone_search_is_empty() {
        assert!(filter_timezones("Not/AZone").is_empty());
    }
}
