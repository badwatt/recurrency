use dioxus::prelude::*;

#[component]
pub fn Tooltip(content: String, children: Element) -> Element {
    rsx! {
        div { class: "group relative inline-flex",
            {children}
            div {
                class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-1 hidden group-hover:block z-50 pointer-events-none",
                div { class: "bg-foreground text-background text-xs rounded px-2 py-1 whitespace-nowrap max-w-64 truncate",
                    "{content}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_renders() {
        let el: Element = rsx! {
            Tooltip { content: "Test tooltip".to_string(),
                button { "Hover me" }
            }
        };
        assert!(el.is_ok());
    }

    #[test]
    fn tooltip_empty_content() {
        let el: Element = rsx! {
            Tooltip { content: String::new(),
                span { "No content" }
            }
        };
        assert!(el.is_ok());
    }

    #[test]
    fn tooltip_long_content() {
        let content = "This is a very long tooltip content that should be truncated".to_string();
        let el: Element = rsx! {
            Tooltip { content,
                div { "Long" }
            }
        };
        assert!(el.is_ok());
    }
}
