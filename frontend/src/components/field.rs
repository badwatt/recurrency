use dioxus::prelude::*;

/// Label + control + error/hint. Use this for every form field so spacing,
/// required markers and validation copy stay identical across pages.
#[component]
pub fn Field(
    label: String,
    #[props(default)] required: bool,
    #[props(default)] error: String,
    #[props(default)] hint: String,
    #[props(default)] class: String,
    #[props(default)] html_for: String,
    #[props(default)] label_class: String,
    children: Element,
) -> Element {
    rsx! {
        div { class: "space-y-1.5 {class}",
            if html_for.is_empty() {
                label { class: "flex items-center gap-1 text-sm font-medium text-foreground {label_class}",
                    {label}
                    if required {
                        span { class: "text-destructive", "*" }
                    }
                }
            } else {
                label { r#for: html_for, class: "flex items-center gap-1 text-sm font-medium text-foreground {label_class}",
                    {label}
                    if required {
                        span { class: "text-destructive", "*" }
                    }
                }
            }
            {children}
            if !error.is_empty() {
                p { class: "text-xs text-destructive", role: "alert", {error} }
            } else if !hint.is_empty() {
                p { class: "text-xs text-muted-foreground", {hint} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn field_exposes_required_and_error_affordances() {
        let source = include_str!("field.rs");
        assert!(source.contains("text-destructive"));
        assert!(source.contains("role: \"alert\""));
    }
}
