use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SegmentVariant {
    #[default]
    Pill,
    Underline,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentOption {
    pub value: String,
    pub label: String,
}

impl SegmentOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// Accessible segmented control used for view switches and tab-like filters.
#[component]
pub fn SegmentedControl(
    value: String,
    options: Vec<SegmentOption>,
    on_change: EventHandler<String>,
    #[props(default)] aria_label: String,
    #[props(default)] class: String,
    #[props(default)] variant: SegmentVariant,
) -> Element {
    let (container, active, idle) = match variant {
        SegmentVariant::Pill => (
            "inline-flex items-center gap-1 rounded-lg border border-border/60 bg-muted/40 p-1",
            "inline-flex flex-1 items-center justify-center gap-2 rounded-md bg-background px-3 py-1.5 text-sm font-semibold text-foreground shadow-sm",
            "inline-flex flex-1 items-center justify-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium text-muted-foreground hover:text-foreground",
        ),
        SegmentVariant::Underline => (
            "flex items-center gap-1 border-b border-border/50",
            "inline-flex items-center gap-2 border-b-2 border-primary px-3 py-2 text-sm font-semibold text-foreground",
            "inline-flex items-center gap-2 border-b-2 border-transparent px-3 py-2 text-sm font-medium text-muted-foreground hover:text-foreground",
        ),
    };
    rsx! {
        div {
            role: "tablist",
            aria_label,
            class: "{container} {class}",
            for option in options {
                {
                    let selected = option.value == value;
                    let option_value = option.value.clone();
                    let option_label = option.label.clone();
                    rsx! {
                        button {
                            key: "{option.value}",
                            r#type: "button",
                            role: "tab",
                            aria_selected: if selected { "true" } else { "false" },
                            class: if selected { active } else { idle },
                            onclick: move |_| on_change.call(option_value.clone()),
                            {option_label}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_option_keeps_value_and_label() {
        let option = SegmentOption::new("accounts", "Accounts");
        assert_eq!(option.value, "accounts");
        assert_eq!(option.label, "Accounts");
    }
}
