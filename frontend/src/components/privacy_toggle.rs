use crate::state::privacy::{PRIVACY, toggle};
use dioxus::prelude::*;

/// Eye button: blurs or reveals all monetary amounts.
#[component]
pub fn PrivacyToggle() -> Element {
    let hidden = *PRIVACY.read();
    rsx! {
        button {
            r#type: "button",
            aria_label: if hidden { "Show amounts" } else { "Hide amounts" },
            title: if hidden { "Show amounts" } else { "Hide amounts" },
            class: "inline-flex h-9 w-9 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground transition-colors",
            onclick: move |_| toggle(),
            if hidden {
                svg {
                    view_box: "0 0 24 24", width: "18", height: "18", fill: "none",
                    stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                    path { d: "M3 3l18 18" }
                    path { d: "M10.6 10.6a2 2 0 0 0 2.8 2.8" }
                    path { d: "M9.9 4.2A10.8 10.8 0 0 1 12 4c7 0 10 8 10 8a17.8 17.8 0 0 1-2.1 3.4" }
                    path { d: "M6.1 6.1A17.7 17.7 0 0 0 2 12s3 8 10 8a10.9 10.9 0 0 0 5.2-1.3" }
                }
            } else {
                svg {
                    view_box: "0 0 24 24", width: "18", height: "18", fill: "none",
                    stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                    path { d: "M2 12s3-8 10-8 10 8 10 8-3 8-10 8S2 12 2 12Z" }
                    circle { cx: "12", cy: "12", r: "3" }
                }
            }
        }
    }
}
