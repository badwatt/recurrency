use dioxus::prelude::*;

/// Accessible switch. One implementation means every on/off control uses the
/// same hit area, focus ring and transition.
#[component]
pub fn Toggle(
    checked: bool,
    on_change: EventHandler<bool>,
    label: String,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
) -> Element {
    let track_class = if checked {
        "relative inline-flex h-6 w-11 shrink-0 items-center rounded-full bg-primary transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 {class}"
    } else {
        "relative inline-flex h-6 w-11 shrink-0 items-center rounded-full bg-muted transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 {class}"
    };
    let knob_class = if checked {
        "inline-block h-4 w-4 translate-x-6 rounded-full bg-white transition-transform"
    } else {
        "inline-block h-4 w-4 translate-x-1 rounded-full bg-white transition-transform"
    };
    rsx! {
        button {
            r#type: "button",
            role: "switch",
            aria_checked: if checked { "true" } else { "false" },
            aria_label: label.clone(),
            title: label,
            disabled,
            class: "{track_class}",
            onclick: move |_| on_change.call(!checked),
            span { class: "{knob_class}" }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn toggle_exposes_switch_semantics() {
        let source = include_str!("toggle.rs");
        assert!(source.contains("role: \"switch\""));
        assert!(source.contains("aria_checked"));
    }
}
