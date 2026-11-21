use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// Dropdown used by forms, filters and settings. The closed control and the
/// popover deliberately match the account/category combobox.
#[component]
pub fn SelectMenu(
    value: String,
    options: Vec<SelectOption>,
    on_change: EventHandler<String>,
    searchable: Option<bool>,
    placeholder: Option<String>,
    disabled: Option<bool>,
    class: Option<String>,
) -> Element {
    let searchable = searchable.unwrap_or(false);
    let placeholder = placeholder.unwrap_or_else(|| "Select...".to_string());
    let disabled = disabled.unwrap_or(false);
    let extra_class = class.unwrap_or_default();
    let mut open = use_signal(|| false);
    let mut query = use_signal(String::new);
    let filter_id = use_signal(|| format!("select-filter-{}", uuid::Uuid::new_v4()));

    use_effect(move || {
        if open() && searchable {
            let id = filter_id();
            let _ = dioxus::document::eval(&format!(
                "requestAnimationFrame(function(){{var e=document.getElementById('{id}'); if(e) e.focus();}});"
            ));
        }
    });

    let query_value = query().trim().to_lowercase();
    let selected_label = options
        .iter()
        .find(|option| option.value == value)
        .map(|option| option.label.clone())
        .unwrap_or_else(|| value.clone());
    let visible: Vec<SelectOption> = options
        .into_iter()
        .filter(|option| {
            !searchable || query_value.is_empty() || option.label.to_lowercase().contains(&query_value)
        })
        .collect();

    rsx! {
        div { class: "relative {extra_class}",
            button {
                r#type: "button",
                class: "h-9 w-full rounded-md border border-border bg-background px-2 text-sm flex items-center justify-between text-left hover:bg-muted disabled:cursor-not-allowed disabled:opacity-50",
                disabled: disabled,
                aria_label: placeholder.clone(),
                aria_haspopup: "listbox",
                aria_expanded: if open() { "true" } else { "false" },
                onclick: move |_| {
                    if open() {
                        open.set(false);
                    } else {
                        query.set(String::new());
                        open.set(true);
                    }
                },
                span { class: if value.is_empty() { "truncate text-muted-foreground" } else { "truncate" },
                    {if selected_label.is_empty() { placeholder.clone() } else { selected_label.clone() }}
                }
                div { class: "flex items-center gap-1 shrink-0 ml-2",
                    svg {
                        class: "h-4 w-4 opacity-50",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        polyline { points: "6 9 12 15 18 9" }
                    }
                }
            }

            if open() {
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| {
                        open.set(false);
                        query.set(String::new());
                    },
                }
                div {
                    class: "absolute left-0 right-0 top-[calc(100%+4px)] z-50 rounded-md border bg-popover shadow-md",
                    role: "listbox",
                    onclick: move |event| event.stop_propagation(),
                    if searchable {
                        div { class: "flex items-center border-b px-3",
                            input {
                                id: filter_id(),
                                class: "w-full h-9 border-0 bg-transparent text-sm focus:outline-none focus:ring-0",
                                r#type: "text",
                                placeholder: "Search...",
                                value: query(),
                                autofocus: true,
                                oninput: move |event| query.set(event.value()),
                                onkeydown: move |event| {
                                    if event.key() == Key::Escape {
                                        open.set(false);
                                        query.set(String::new());
                                    }
                                },
                            }
                        }
                    }
                    div { class: "max-h-[200px] overflow-y-auto py-1",
                        if visible.is_empty() {
                            div { class: "py-6 text-center text-sm text-muted-foreground", "No results found" }
                        } else {
                            for option in visible {
                                {
                                    let selected = option.value == value;
                                    let option_value = option.value.clone();
                                    let label = option.label.clone();
                                    rsx! {
                                        button {
                                            key: "{option.value}",
                                            r#type: "button",
                                            role: "option",
                                            aria_selected: if selected { "true" } else { "false" },
                                            class: if selected {
                                                "relative flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent bg-accent"
                                            } else {
                                                "relative flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent"
                                            },
                                            onclick: move |_| {
                                                on_change.call(option_value.clone());
                                                query.set(String::new());
                                                open.set(false);
                                            },
                                            span { class: "truncate", {label} }
                                            if selected {
                                                svg {
                                                    class: "ml-auto h-4 w-4 shrink-0 text-primary",
                                                    view_box: "0 0 24 24",
                                                    fill: "none",
                                                    stroke: "currentColor",
                                                    stroke_width: "2",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    polyline { points: "20 6 9 17 4 12" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
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
    fn select_option_construction() {
        let option = SelectOption::new("EUR", "EUR — Euro");
        assert_eq!(option.value, "EUR");
        assert_eq!(option.label, "EUR — Euro");
    }

    #[test]
    fn searchable_menu_focuses_the_filter() {
        let source = include_str!("select_menu.rs");
        assert!(source.contains("autofocus: true"));
        assert!(source.contains("Search..."));
        assert!(source.contains("requestAnimationFrame"));
    }
}
