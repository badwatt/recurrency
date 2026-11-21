use dioxus::prelude::*;

/// Shared page header: title, description and an optional right-hand action
/// slot. Animation belongs on the page wrapper, never on the header itself.
#[component]
pub fn PageHeader(
    title: String,
    subtitle: String,
    actions: Option<Element>,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between {class}",
            div { class: "min-w-0",
                h1 { class: "text-4xl font-bold tracking-tighter md:text-5xl", {title} }
                p { class: "text-muted-foreground", {subtitle} }
            }
            if let Some(actions) = actions {
                div { class: "flex shrink-0 flex-wrap items-center gap-2", {actions} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn page_header_has_an_action_slot() {
        let source = include_str!("page_header.rs");
        assert!(source.contains("actions: Option<Element>"));
        assert!(source.contains("sm:items-end"));
    }
}
