use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardRadius {
    #[default]
    Xl,
    Lg,
}

impl CardRadius {
    fn class(self) -> &'static str {
        match self {
            Self::Xl => "rounded-xl",
            Self::Lg => "rounded-lg",
        }
    }
}

/// Shared surface for settings, reports and content panels.
#[component]
pub fn Card(
    #[props(default)] radius: CardRadius,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    rsx! {
        div { class: "{radius.class()} border border-border/40 bg-card/80 text-card-foreground transition-colors duration-300 {class}",
            {children}
        }
    }
}

#[component]
pub fn CardHeader(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "flex flex-col space-y-1.5 p-6 {class}", {children} }
    }
}

#[component]
pub fn CardTitle(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        h3 { class: "text-lg font-semibold leading-none tracking-tight {class}", {children} }
    }
}

#[component]
pub fn CardContent(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "p-6 pt-0 {class}", {children} }
    }
}

/// Card with a standard heading block and a separated body. Settings pages
/// should use this instead of repeating the same header/border markup.
#[component]
pub fn SectionCard(
    title: String,
    #[props(default)] description: String,
    actions: Option<Element>,
    #[props(default)] class: String,
    #[props(default)] title_class: String,
    #[props(default)] header_class: String,
    #[props(default)] content_class: String,
    children: Element,
) -> Element {
    rsx! {
        Card { class: "overflow-hidden {class}",
            div { class: "flex items-start justify-between gap-3 border-b border-border p-6 {header_class}",
                div { class: "min-w-0 space-y-1",
                    h2 { class: "text-lg font-semibold {title_class}", {title} }
                    if !description.is_empty() {
                        p { class: "text-sm text-muted-foreground", {description} }
                    }
                }
                if let Some(actions) = actions {
                    div { class: "shrink-0", {actions} }
                }
            }
            div { class: "p-6 {content_class}", {children} }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn card_uses_the_shared_surface_tokens() {
        let source = include_str!("card.rs");
        assert!(source.contains("rounded-xl border border-border/40 bg-card/80"));
    }

    #[test]
    fn section_card_owns_the_heading_and_body_spacing() {
        let source = include_str!("card.rs");
        assert!(source.contains("border-b border-border p-6"));
        assert!(source.contains("class: \"p-6 {content_class}\""));
    }
}
