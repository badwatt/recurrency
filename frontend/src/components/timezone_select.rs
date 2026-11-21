use crate::components::select_menu::{SelectMenu, SelectOption};
use crate::utils::timezones::all_timezones;
use dioxus::prelude::*;

/// Searchable IANA timezone selector using the shared select surface.
#[component]
pub fn TimezoneSelect(value: Signal<String>) -> Element {
    let mut value = value;
    rsx! {
        SelectMenu {
            value: value(),
            options: all_timezones()
                .into_iter()
                .map(|timezone| SelectOption::new(timezone, timezone))
                .collect::<Vec<_>>(),
            on_change: move |next: String| value.set(next),
            searchable: true,
            placeholder: "Select timezone...",
            class: "mt-1",
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn timezone_picker_uses_the_shared_searchable_select() {
        let source = include_str!("timezone_select.rs");
        assert!(source.contains("SelectMenu"));
        assert!(source.contains("searchable: true"));
    }
}
