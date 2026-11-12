use super::{FilterConfig, SortConfig};
use crate::components::ListToolbar;
use crate::state::categories::CATEGORIES;
use dioxus::prelude::*;
use lucide_dioxus::ArrowUpDown;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

/// Search, filters, sort and view controls for the transactions list.
#[component]
pub fn FilterSortBar(
    filters: Signal<FilterConfig>,
    sort_cfg: Signal<SortConfig>,
    query: Signal<String>,
    view: Signal<String>,
    selection_mode: Signal<bool>,
) -> Element {
    let mut filters_popover = use_signal(|| false);
    let mut sort_popover = use_signal(|| false);
    let cats = CATEGORIES();
    let filter_count = usize::from(filters().category_id.is_some())
        + usize::from(filters().price_min.is_some())
        + usize::from(filters().price_max.is_some());

    let actions = rsx! {
        div { class: "relative",
            button {
                r#type: "button",
                aria_label: "Filters",
                title: "Filters",
                aria_expanded: if filters_popover() { "true" } else { "false" },
                class: if filter_count > 0 { "relative inline-flex h-10 w-10 items-center justify-center rounded-lg border border-primary/40 bg-primary/10 text-primary hover:bg-primary/15" } else { "relative inline-flex h-10 w-10 items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground" },
                onclick: move |_| filters_popover.set(!filters_popover()),
                svg {
                    view_box: "0 0 24 24",
                    width: "16",
                    height: "16",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    path { d: "M4 5h16l-6 7v5l-4 2v-7L4 5z" }
                }
                if filter_count > 0 {
                    span { class: "absolute -right-1 -top-1 min-w-4 rounded-full bg-primary px-1 text-[10px] leading-4 text-primary-foreground", "{filter_count}" }
                }
            }
            if filters_popover() {
                div {
                    class: "absolute right-0 top-full z-50 mt-2 w-72 rounded-lg border bg-popover p-3 shadow-md space-y-3",
                    onclick: move |e| e.stop_propagation(),
                    div { class: "flex items-center justify-between",
                        label { class: "text-sm font-medium", "Filters" }
                        if filter_count > 0 {
                            button {
                                r#type: "button",
                                class: "px-2 py-0.5 text-xs text-muted-foreground hover:text-foreground",
                                onclick: move |_| filters.set(FilterConfig::default()),
                                "Clear"
                            }
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-xs font-medium text-muted-foreground", "Category" }
                        select {
                            class: "h-8 w-full rounded-md border bg-background px-2 text-sm",
                            value: filters().category_id.map(|id| id.to_string()).unwrap_or_default(),
                            onchange: move |e| {
                                let value = e.value();
                                let mut next = filters();
                                next.category_id = if value.is_empty() { None } else { Uuid::parse_str(&value).ok() };
                                filters.set(next);
                            },
                            option { value: "", "All categories" }
                            for cat in &cats.categories {
                                option { value: cat.id.to_string(), {cat.name.clone()} }
                            }
                        }
                    }
                    div { class: "grid grid-cols-2 gap-2",
                        label { class: "space-y-1.5 text-xs font-medium text-muted-foreground", "Min"
                            input {
                                r#type: "number",
                                step: "0.01",
                                placeholder: "0.00",
                                value: filters().price_min.map(|value| value.to_string()).unwrap_or_default(),
                                oninput: move |e| {
                                    let mut next = filters();
                                    next.price_min = Decimal::from_str(&e.value()).ok();
                                    filters.set(next);
                                },
                                class: "h-8 w-full rounded-md border bg-background px-2 text-sm",
                            }
                        }
                        label { class: "space-y-1.5 text-xs font-medium text-muted-foreground", "Max"
                            input {
                                r#type: "number",
                                step: "0.01",
                                placeholder: "999.99",
                                value: filters().price_max.map(|value| value.to_string()).unwrap_or_default(),
                                oninput: move |e| {
                                    let mut next = filters();
                                    next.price_max = Decimal::from_str(&e.value()).ok();
                                    filters.set(next);
                                },
                                class: "h-8 w-full rounded-md border bg-background px-2 text-sm",
                            }
                        }
                    }
                }
                div { class: "fixed inset-0 z-40", onclick: move |_| filters_popover.set(false) }
            }
        }
        div { class: "relative",
            button {
                r#type: "button",
                aria_label: "Sort",
                title: "Sort",
                aria_expanded: if sort_popover() { "true" } else { "false" },
                class: "inline-flex h-10 w-10 items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground",
                onclick: move |_| sort_popover.set(!sort_popover()),
                ArrowUpDown { size: 16 }
            }
            if sort_popover() {
                div {
                    class: "absolute right-0 top-full z-50 mt-2 w-52 rounded-lg border bg-popover p-3 shadow-md space-y-2",
                    onclick: move |e| e.stop_propagation(),
                    label { class: "text-xs font-medium text-muted-foreground", "Sort by" }
                    select {
                        class: "h-8 w-full rounded-md border bg-background px-2 text-sm",
                        value: "{sort_cfg().field}",
                        onchange: move |e| {
                            let mut next = sort_cfg();
                            next.field = e.value();
                            sort_cfg.set(next);
                        },
                        option { value: "date", "Date" }
                        option { value: "amount", "Price" }
                        option { value: "name", "Name" }
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex h-8 w-full items-center justify-between rounded-md border bg-background px-2 text-sm",
                        onclick: move |_| {
                            let mut next = sort_cfg();
                            next.direction = if next.direction == "desc" { "asc".into() } else { "desc".into() };
                            sort_cfg.set(next);
                        },
                        span { if sort_cfg().direction == "desc" { "Descending" } else { "Ascending" } }
                        span { if sort_cfg().direction == "desc" { "↓" } else { "↑" } }
                    }
                }
                div { class: "fixed inset-0 z-40", onclick: move |_| sort_popover.set(false) }
            }
        }
        button {
            r#type: "button",
            aria_label: if selection_mode() { "Exit selection mode" } else { "Enter selection mode" },
            aria_pressed: if selection_mode() { "true" } else { "false" },
            class: if selection_mode() { "inline-flex h-10 w-10 items-center justify-center rounded-lg border border-primary/40 bg-primary/10 text-primary" } else { "inline-flex h-10 w-10 items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground" },
            onclick: move |_| selection_mode.toggle(),
            svg {
                view_box: "0 0 24 24",
                width: "16",
                height: "16",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                rect { x: "3", y: "3", width: "18", height: "18", rx: "3" }
                if selection_mode() {
                    path { d: "m7 12 3 3 7-7" }
                }
            }
        }
    };

    rsx! {
        ListToolbar {
            value: query(),
            oninput: move |value| query.set(value),
            view: view(),
            onview: move |value| view.set(value),
            show_view: true,
            actions: actions,
        }
    }
}
