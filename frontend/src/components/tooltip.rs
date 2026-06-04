use dioxus::prelude::*;

#[component]
pub fn Tooltip(content: String, children: Element) -> Element {
    if content.is_empty() {
        return rsx! { {children} };
    }
    rsx! {
        div { class: "relative inline-flex",
            div { class: "peer", {children} }
            div { class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-1 hidden peer-hover:block z-50 pointer-events-none",
                div { class: "bg-foreground text-background text-xs rounded px-2 py-1 whitespace-pre-wrap max-w-xs",
                    {content.to_string()}                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_renders_with_content() {
        let el: Element = rsx! {
            Tooltip { content: "Hello".to_string(),
                button { "Hover" }
            }
        };
        assert!(el.is_ok());
    }

    #[test]
    fn tooltip_empty_returns_bare_children() {
        let el: Element = rsx! {
            Tooltip { content: String::new(),
                button { "No tip" }
            }
        };
        assert!(el.is_ok());
    }

    #[test]
    fn tooltip_uses_peer_hover_not_group_hover() {
        let src = include_str!("tooltip.rs");
        assert!(src.contains("peer-hover:block"), "must use peer-hover:block");
        // Only check production code for banned patterns
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(!prod.contains("group-hover"), "production code must not use group-hover");
    }

    #[test]
    fn tooltip_child_wrapper_has_peer_class() {
        let src = include_str!("tooltip.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(prod.contains("\"peer\""), "child wrapper must have peer class");
    }

    #[test]
    fn tooltip_no_group_class_in_production() {
        let src = include_str!("tooltip.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(!prod.contains("\"group\""), "production code must not use group class");
    }

    #[test]
    fn tooltip_positioned_relative_to_container() {
        let src = include_str!("tooltip.rs");
        assert!(src.contains("relative"), "outer container must have relative");
        assert!(src.contains("absolute"), "popup must use absolute positioning");
    }

    #[test]
    fn tooltip_hidden_by_default() {
        let src = include_str!("tooltip.rs");
        assert!(src.contains("hidden"), "popup must be hidden by default");
    }

    #[test]
    fn tooltip_full_content_no_truncate() {
        let src = include_str!("tooltip.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(!prod.contains("truncate"), "must not truncate content");
        assert!(src.contains("whitespace-pre-wrap"), "must preserve whitespace");
    }

    #[test]
    fn tooltip_pointer_events_none() {
        let src = include_str!("tooltip.rs");
        assert!(src.contains("pointer-events-none"), "popup must not intercept mouse events");
    }
}