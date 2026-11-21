use crate::components::theme_picker::ThemeGrid;
use dioxus::prelude::*;

/// Theme selection. Themes are stored on the device so the pre-WASM loader
/// can apply the same palette before the app starts.
#[component]
pub fn AppearanceCard() -> Element {
    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Appearance" }
                p { class: "text-sm text-muted-foreground",
                    "Choose a palette. It applies immediately and is saved on this device."
                }
            }
            div { class: "p-6",
                ThemeGrid {}
            }
        }
    }
}
