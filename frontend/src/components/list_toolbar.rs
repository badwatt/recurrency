//! Compact list toolbar: search plus icon-only filters, sort and view controls.

use dioxus::prelude::*;
use lucide_dioxus::{Search, X};

#[component]
pub fn ListToolbar(
    value: String,
    oninput: EventHandler<String>,
    view: String,
    onview: EventHandler<String>,
    actions: Element,
) -> Element {
    let is_grid = view == "grid";
    rsx! {
        div { role: "search", class: "flex flex-col gap-2 sm:flex-row sm:items-center", aria_label: "Search and filters",
            div { class: "relative min-w-0 flex-1",
                span { class: "pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground",
                    Search { size: 16 }
                }
                input {
                    r#type: "search",
                    value: value,
                    placeholder: "Search",
                    aria_label: "Search",
                    oninput: move |e| oninput.call(e.value()),
                    autocomplete: "off",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    enterkeyhint: "search",
                    class: "h-10 w-full rounded-lg border border-border/60 bg-background pl-9 pr-9 text-sm outline-none transition-colors focus:border-primary/50 focus-visible:ring-2 focus-visible:ring-primary/40 focus-visible:ring-offset-2 focus-visible:ring-offset-background",
                }
                if !value.is_empty() {
                    button {
                        r#type: "button",
                        aria_label: "Clear search",
                        class: "absolute right-2 top-1/2 -translate-y-1/2 inline-flex h-6 w-6 items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground",
                        onclick: move |_| oninput.call(String::new()),
                        X { size: 14 }
                    }
                }
            }
            div { class: "flex flex-wrap items-center gap-2 sm:justify-end",
                {actions}
                div {
                    role: "group",
                    aria_label: "View mode",
                    class: "inline-flex h-10 items-center rounded-lg border border-border/60 bg-muted/40 p-0.5",
                    button {
                        r#type: "button",
                        aria_label: "Grid view",
                        title: "Grid view",
                        aria_pressed: if is_grid { "true" } else { "false" },
                        class: if is_grid { "inline-flex h-9 w-9 items-center justify-center rounded-md bg-background text-foreground shadow-sm" } else { "inline-flex h-9 w-9 items-center justify-center rounded-md text-muted-foreground hover:text-foreground" },
                        onclick: move |_| onview.call("grid".to_string()),
                        svg {
                            view_box: "0 0 24 24",
                            width: "16",
                            height: "16",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
                            rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
                            rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
                            rect { x: "14", y: "14", width: "7", height: "7", rx: "1" }
                        }
                    }
                    button {
                        r#type: "button",
                        aria_label: "List view",
                        title: "List view",
                        aria_pressed: if is_grid { "false" } else { "true" },
                        class: if is_grid { "inline-flex h-9 w-9 items-center justify-center rounded-md text-muted-foreground hover:text-foreground" } else { "inline-flex h-9 w-9 items-center justify-center rounded-md bg-background text-foreground shadow-sm" },
                        onclick: move |_| onview.call("list".to_string()),
                        svg {
                            view_box: "0 0 24 24",
                            width: "16",
                            height: "16",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            line { x1: "4", y1: "6", x2: "20", y2: "6" }
                            line { x1: "4", y1: "12", x2: "20", y2: "12" }
                            line { x1: "4", y1: "18", x2: "20", y2: "18" }
                        }
                    }
                }
            }
        }
    }
}
