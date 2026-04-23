use dioxus::prelude::*;

#[component]
pub fn Tooltip(content: String, children: Element) -> Element {
    if content.is_empty() {
        return rsx! { {children} };
    }
    rsx! {
        div { class: "relative inline-block",
            div { class: "group", {children} }
            div { class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-1 hidden group-hover:block z-50 pointer-events-none",
                div { class: "bg-foreground text-background text-xs rounded px-2 py-1 whitespace-pre-wrap max-w-xs",
                    "{content}"
                }
            }
        }
    }
}
