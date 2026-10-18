use super::{FilterConfig, SortConfig};
use crate::state::categories::CATEGORIES;
use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

/// Filters popover + sort controls for the transactions list.
#[component]
pub fn FilterSortBar(
    filters: Signal<FilterConfig>,
    sort_cfg: Signal<SortConfig>,
) -> Element {
    let mut filters_popover = use_signal(|| false);
    let cats = CATEGORIES();

    rsx! {
        div { class: "flex flex-col sm:flex-row items-start sm:items-center gap-2",
            // Filters button + popover
            div { class: "relative",
                button {
                    class: "border px-3 py-1.5 rounded-md text-sm flex items-center gap-1.5 hover:bg-muted h-8",
                    onclick: move |_| filters_popover.set(!filters_popover()),
                    "▾ Filters"
                    if filters().category_id.is_some() || filters().price_min.is_some() || filters().price_max.is_some() {
                        span { class: "ml-1 bg-secondary text-secondary-foreground rounded-full px-1.5 text-[10px]", "1" }
                    }
                }
                if filters_popover() {
                    div { class: "absolute left-0 top-full mt-2 z-50 w-72 rounded-md border bg-popover p-3 shadow-md space-y-3",
                        onclick: move |e| e.stop_propagation(),
                        div { class: "flex items-center justify-between",
                            label { class: "text-sm font-medium", "Filters" }
                            if filters().category_id.is_some() || filters().price_min.is_some() || filters().price_max.is_some() {
                                button {
                                    class: "text-xs text-muted-foreground hover:text-foreground px-2 py-0.5",
                                    onclick: move |_| filters.set(FilterConfig::default()),
                                    "Clear"
                                }
                            }
                        }
                        // Category filter
                        div { class: "space-y-1.5",
                            label { class: "text-xs font-medium text-muted-foreground", "Category" }
                            select {
                                class: "w-full h-8 rounded-md border bg-background px-2 text-sm",
                                value: filters().category_id.map(|id| id.to_string()).unwrap_or_default(),
                                onchange: move |e| {
                                    let val = e.value();
                                    let id = if val.is_empty() { None } else { Uuid::parse_str(&val).ok() };
                                    let mut f = filters();
                                    f.category_id = id;
                                    filters.set(f);
                                },
                                option { value: "", "All categories" }
                                for cat in &cats.categories {
                                    option { value: cat.id.to_string(), {cat.name.to_string()} }
                                }
                            }
                        }
                        // Min price
                        div { class: "space-y-1.5",
                            label { class: "text-xs font-medium text-muted-foreground", "Min price" }
                            input {
                                r#type: "number",
                                step: "0.01",
                                placeholder: "0.00",
                                value: filters().price_min.map(|d| d.to_string()).unwrap_or_default(),
                                oninput: move |e| {
                                    let mut f = filters();
                                    f.price_min = Decimal::from_str(&e.value()).ok();
                                    filters.set(f);
                                },
                                class: "w-full h-8 rounded-md border bg-background px-3 py-1 text-sm",
                            }
                        }
                        // Max price
                        div { class: "space-y-1.5",
                            label { class: "text-xs font-medium text-muted-foreground", "Max price" }
                            input {
                                r#type: "number",
                                step: "0.01",
                                placeholder: "999.99",
                                value: filters().price_max.map(|d| d.to_string()).unwrap_or_default(),
                                oninput: move |e| {
                                    let mut f = filters();
                                    f.price_max = Decimal::from_str(&e.value()).ok();
                                    filters.set(f);
                                },
                                class: "w-full h-8 rounded-md border bg-background px-3 py-1 text-sm",
                            }
                        }
                    }
                }
            }

            // Sort section
            div { class: "flex items-center gap-1.5 w-full sm:w-auto sm:ml-auto",
                select {
                    class: "h-8 w-full sm:w-[120px] rounded-md border bg-background px-2 text-sm",
                    value: "{sort_cfg().field}",
                    onchange: move |e| {
                        let mut s = sort_cfg();
                        s.field = e.value();
                        sort_cfg.set(s);
                    },
                    option { value: "date", "Date" }
                    option { value: "amount", "Price" }
                    option { value: "name", "Name" }
                }
                button {
                    class: "h-8 w-8 flex items-center justify-center rounded-md border bg-background hover:bg-muted",
                    onclick: move |_| {
                        let mut s = sort_cfg();
                        s.direction = if s.direction == "desc" { "asc".to_string() } else { "desc".to_string() };
                        sort_cfg.set(s);
                    },
                    if sort_cfg().direction == "desc" { "↓" } else { "↑" }
                }
            }
        }
    }
}
