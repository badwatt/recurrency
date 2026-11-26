//! Searchable IANA timezone picker shared by every schedule form.

use crate::components::string_select::{StringSelectProps, string_select};
use nsh_core::prelude::*;

pub fn timezone_select(value: Signal<String>) -> impl IntoElement {
    let mut options = chrono_tz::TZ_VARIANTS
        .iter()
        .map(|zone| (zone.name().to_string(), zone.name().to_string()))
        .collect::<Vec<_>>();
    options.sort_by(|a, b| a.0.cmp(&b.0));
    string_select(StringSelectProps::new("Timezone", value, options))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_timezone_catalogue_is_not_empty_and_contains_utc() {
        let options = chrono_tz::TZ_VARIANTS
            .iter()
            .map(|zone| zone.name().to_string())
            .collect::<Vec<_>>();
        assert!(options.iter().any(|zone| zone == "UTC"));
        assert!(options.len() > 100);
    }
}
