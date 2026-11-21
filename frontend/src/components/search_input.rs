use dioxus::prelude::*;
use lucide_dioxus::{Search, X};

#[component]
pub fn SearchInput(
    value: String,
    on_input: EventHandler<String>,
    #[props(default = "Search".to_string())] placeholder: String,
    #[props(default)] aria_label: String,
    #[props(default)] class: String,
) -> Element {
    let label = if aria_label.is_empty() {
        placeholder.clone()
    } else {
        aria_label
    };
    let field_id = use_hook(|| format!("search-{}", uuid::Uuid::new_v4()));
    rsx! {
        div { class: "relative min-w-0 {class}",
            span { class: "pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground",
                Search { size: 16 }
            }
            input {
                r#type: "text",
                id: field_id,
                name: "search",
                value,
                placeholder,
                aria_label: label,
                oninput: move |event| on_input.call(event.value()),
                autocomplete: "off",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                enterkeyhint: "search",
                class: "h-10 w-full rounded-lg border border-border/60 bg-background pl-9 pr-9 text-sm outline-none transition-colors focus:border-primary/50",
            }
            if !value.is_empty() {
                button {
                    r#type: "button",
                    aria_label: "Clear search",
                    class: "absolute right-2 top-1/2 inline-flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground",
                    onclick: move |_| on_input.call(String::new()),
                    X { size: 14 }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn search_input_keeps_clear_and_search_affordances() {
        let source = include_str!("search_input.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(source.contains("Clear search"));
        assert!(source.contains("r#type: \"text\""));
        assert!(!production.contains("focus-visible:ring"));
        assert!(!production.contains("--tw-ring-shadow"));
    }
}
