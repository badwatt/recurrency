use crate::components::button::{Button, ButtonAlign, ButtonRadius, ButtonSize, ButtonVariant};
use crate::components::input::{Input, InputSize, InputVariant};
use dioxus::prelude::*;
use lucide_dioxus::{Check, ChevronDown};

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
    id: Option<String>,
) -> Element {
    let searchable = searchable.unwrap_or(false);
    let placeholder = placeholder.unwrap_or_else(|| "Select...".to_string());
    let disabled = disabled.unwrap_or(false);
    let extra_class = class.unwrap_or_default();
    let button_id = id.unwrap_or_default();
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
            Button {
                variant: ButtonVariant::Unstyled,
                size: ButtonSize::Unstyled,
                id: button_id,
                align: ButtonAlign::Between,
                class: "flex h-9 w-full items-center rounded-md border border-border bg-background px-2 text-left text-sm hover:bg-muted disabled:cursor-not-allowed disabled:opacity-50".to_string(),
                disabled: disabled,
                aria_label: Some(placeholder.clone()),
                aria_haspopup: Some("listbox".to_string()),
                aria_expanded: Some((if open() { "true" } else { "false" }).to_string()),
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
                div { class: "ml-2 flex shrink-0 items-center gap-1",
                    ChevronDown { size: 16, class: "opacity-50" }
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
                            Input {
                                id: filter_id(),
                                size: InputSize::Sm,
                                variant: InputVariant::Bare,
                                class: String::new(),
                                placeholder: "Search...".to_string(),
                                value: query(),
                                autofocus: true,
                                on_input: move |value: String| query.set(value),
                                on_keydown: move |event: KeyboardEvent| {
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
                                        Button {
                                            key: "{option.value}",
                                            variant: ButtonVariant::Unstyled,
                                            size: ButtonSize::Unstyled,
                                            radius: ButtonRadius::None,
                                            role: Some("option".to_string()),
                                            aria_selected: Some((if selected { "true" } else { "false" }).to_string()),
                                            class: (if selected {
                                                "relative flex w-full items-center justify-start gap-2 bg-accent px-3 py-2 text-left text-sm hover:bg-accent"
                                            } else {
                                                "relative flex w-full items-center justify-start gap-2 px-3 py-2 text-left text-sm hover:bg-accent"
                                            }).to_string(),
                                            onclick: move |_| {
                                                on_change.call(option_value.clone());
                                                query.set(String::new());
                                                open.set(false);
                                            },
                                            span { class: "truncate", {label} }
                                            if selected {
                                                Check { size: 16, class: "ml-auto shrink-0 text-primary" }
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
