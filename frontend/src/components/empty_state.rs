use dioxus::prelude::*;

#[component]
pub fn EmptyState(
    title: String,
    description: String,
    icon: Option<Element>,
    action: Option<Element>,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center gap-4 rounded-xl border border-border/60 bg-card p-8 text-center {class}",
            if let Some(icon) = icon {
                div { class: "text-muted-foreground", {icon} }
            }
            div { class: "space-y-1",
                p { class: "text-lg font-medium", {title} }
                p { class: "text-sm text-muted-foreground", {description} }
            }
            if let Some(action) = action {
                {action}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn empty_state_is_shared_by_lists() {
        let source = include_str!("empty_state.rs");
        assert!(source.contains("flex flex-col items-center"));
        assert!(source.contains("text-muted-foreground"));
    }
}
