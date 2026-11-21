use crate::utils::timezones::filter_timezones;
use dioxus::prelude::*;

/// Searchable IANA timezone selector.
#[component]
pub fn TimezoneSelect(value: Signal<String>) -> Element {
    let mut value = value;
    let mut open = use_signal(|| false);
    let mut query = use_signal(String::new);
    let current = value();
    let options = filter_timezones(&query());

    rsx! {
        div { class: "relative",
            button {
                r#type: "button",
                class: "mt-1 flex h-9 w-full items-center justify-between rounded-md border border-input bg-background px-3 text-sm text-foreground",
                aria_label: "Select timezone",
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
                span { class: "truncate", {if current.is_empty() { "Select timezone".to_string() } else { current.clone() }} }
                svg {
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    class: "h-4 w-4 shrink-0 opacity-50",
                    polyline { points: "6 9 12 15 18 9" }
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
                    class: "absolute left-0 right-0 top-full z-50 mt-1 overflow-hidden rounded-md border border-border bg-popover shadow-lg",
                    role: "listbox",
                    aria_label: "Timezones",
                    onclick: move |event| event.stop_propagation(),
                    div { class: "border-b border-border/60 p-2",
                        input {
                            r#type: "text",
                            value: query(),
                            oninput: move |event| query.set(event.value()),
                            placeholder: "Search timezones...",
                            autofocus: true,
                            class: "h-9 w-full rounded-md border border-input bg-background px-3 text-sm",
                        }
                    }
                    div { class: "max-h-64 overflow-y-auto p-1",
                        if options.is_empty() {
                            p { class: "px-3 py-4 text-center text-sm text-muted-foreground", "No timezones found" }
                        } else {
                            for timezone in options {
                                button {
                                    key: "{timezone}",
                                    r#type: "button",
                                    role: "option",
                                    aria_selected: if current == timezone { "true" } else { "false" },
                                    class: if current == timezone {
                                        "flex w-full items-center justify-between rounded px-3 py-2 text-left text-sm bg-primary/10 text-foreground"
                                    } else {
                                        "flex w-full items-center justify-between rounded px-3 py-2 text-left text-sm text-muted-foreground hover:bg-accent hover:text-foreground"
                                    },
                                    onclick: move |_| {
                                        value.set(timezone.to_string());
                                        query.set(String::new());
                                        open.set(false);
                                    },
                                    span { class: "truncate", {timezone} }
                                    if current == timezone {
                                        span { class: "text-[10px] font-medium text-primary", "Selected" }
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
    #[test]
    fn timezone_component_exposes_search_and_a_listbox() {
        let source = include_str!("timezone_select.rs");
        assert!(source.contains("Search timezones..."));
        assert!(source.contains("role: \"listbox\""));
        assert!(source.contains("filter_timezones"));
    }
}
