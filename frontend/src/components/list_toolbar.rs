//! Compact list toolbar: shared search plus icon-only view controls.

use crate::components::button::{ButtonAlign, ButtonSize, ButtonVariant, IconButton};
use crate::components::search_input::SearchInput;
use dioxus::prelude::*;

#[component]
pub fn ListToolbar(
    value: String,
    oninput: EventHandler<String>,
    view: String,
    onview: EventHandler<String>,
    show_view: bool,
    actions: Element,
) -> Element {
    let is_grid = view == "grid";
    rsx! {
        div { role: "search", class: "flex flex-col gap-2 sm:flex-row sm:items-center", aria_label: "Search and filters",
            SearchInput {
                class: "flex-1".to_string(),
                value,
                on_input: move |value| oninput.call(value),
            }
            div { class: "relative flex w-full flex-wrap items-center gap-2 sm:w-auto sm:justify-end",
                {actions}
                if show_view {
                    div { role: "group", aria_label: "View mode", class: "inline-flex h-10 items-center rounded-lg border border-border/60 bg-muted/40 p-0.5",
                        IconButton {
                            label: "Grid view".to_string(),
                            variant: ButtonVariant::Unstyled,
                            size: ButtonSize::Icon,
                            align: ButtonAlign::Center,
                            aria_pressed: Some(if is_grid { "true" } else { "false" }.to_string()),
                            class: (if is_grid { "inline-flex items-center justify-center bg-background text-foreground shadow-sm" } else { "inline-flex items-center justify-center text-muted-foreground hover:text-foreground" }).to_string(),
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
                        IconButton {
                            label: "List view".to_string(),
                            variant: ButtonVariant::Unstyled,
                            size: ButtonSize::Icon,
                            align: ButtonAlign::Center,
                            aria_pressed: Some(if is_grid { "false" } else { "true" }.to_string()),
                            class: (if is_grid { "inline-flex items-center justify-center text-muted-foreground hover:text-foreground" } else { "inline-flex items-center justify-center bg-background text-foreground shadow-sm" }).to_string(),
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn toolbar_uses_the_shared_search_input() {
        let source = include_str!("list_toolbar.rs");
        assert!(source.contains("SearchInput {"));
        assert!(source.contains("IconButton {"));
    }
}
