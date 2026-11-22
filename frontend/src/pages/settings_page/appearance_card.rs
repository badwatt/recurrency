use crate::components::theme_picker::ThemeGrid;
use dioxus::prelude::*;

/// Theme selection. Themes are stored on the device so the pre-WASM loader
/// can apply the same palette before the app starts.
use crate::components::card::SectionCard;
#[component]
pub fn AppearanceCard() -> Element {
    rsx! {
        SectionCard {
            title: "Color themes",
            description: "Choose a palette. It applies immediately and syncs across devices when connected.",
            content_class: "p-0",
            div { class: "p-6",
                ThemeGrid {}
            }
        }
    }
}
