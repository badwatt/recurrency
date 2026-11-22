use super::{FilterConfig, SortConfig};
use crate::components::ListToolbar;
use crate::components::button::{Button, ButtonAlign, ButtonRadius, ButtonSize, ButtonVariant};
use crate::components::field::Field;
use crate::components::input::{Input, InputSize};
use crate::components::popover::{Popover, PopoverAlign};
use crate::components::select_menu::{SelectMenu, SelectOption};
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
        div { class: "static sm:relative",
            Button {
                variant: ButtonVariant::Unstyled,
                size: ButtonSize::IconLg,
                radius: ButtonRadius::Lg,
                align: ButtonAlign::Center,
                aria_label: Some("Filters".to_string()),
                title: Some("Filters".to_string()),
                aria_expanded: Some((if filters_popover() { "true" } else { "false" }).to_string()),
                class: (if filter_count > 0 { "relative inline-flex items-center justify-center rounded-lg border border-primary/40 bg-primary/10 text-primary hover:bg-primary/15" } else { "relative inline-flex items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground" }).to_string(),
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
            Popover {
                open: filters_popover(),
                on_close: move |_| filters_popover.set(false),
                align: PopoverAlign::CenterSm,
                class: "w-[min(18rem,calc(100vw-2rem))] space-y-3 p-3".to_string(),
                div { class: "flex items-center justify-between",
                    label { class: "text-sm font-medium", "Filters" }
                    if filter_count > 0 {
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Xs,
                            class: "text-muted-foreground".to_string(),
                            onclick: move |_| filters.set(FilterConfig::default()),
                            "Clear"
                        }
                    }
                }
                Field { label: "Category", label_class: "text-xs text-muted-foreground".to_string(),
                    SelectMenu {
                        value: filters().category_id.map(|id| id.to_string()).unwrap_or_default(),
                        options: {
                            let mut options = vec![SelectOption::new("", "All categories")];
                            options.extend(cats.categories.iter().map(|cat| SelectOption::new(cat.id.to_string(), cat.name.clone())));
                            options
                        },
                        on_change: move |value: String| {
                            let mut next = filters();
                            next.category_id = if value.is_empty() { None } else { Uuid::parse_str(&value).ok() };
                            filters.set(next);
                        },
                        searchable: true,
                        placeholder: "All categories",
                    }
                }
                div { class: "grid grid-cols-2 gap-2",
                    Field { label: "Min", label_class: "text-xs text-muted-foreground".to_string(),
                        Input {
                            r#type: "number",
                            size: InputSize::Xs,
                            step: "0.01",
                            placeholder: "0.00",
                            value: filters().price_min.map(|value| value.to_string()).unwrap_or_default(),
                            on_input: move |value: String| {
                                let mut next = filters();
                                next.price_min = Decimal::from_str(&value).ok();
                                filters.set(next);
                            },
                        }
                    }
                    Field { label: "Max", label_class: "text-xs text-muted-foreground".to_string(),
                        Input {
                            r#type: "number",
                            size: InputSize::Xs,
                            step: "0.01",
                            placeholder: "999.99",
                            value: filters().price_max.map(|value| value.to_string()).unwrap_or_default(),
                            on_input: move |value: String| {
                                let mut next = filters();
                                next.price_max = Decimal::from_str(&value).ok();
                                filters.set(next);
                            },
                        }
                    }
                }
            }
        }
        div { class: "static sm:relative",
            Button {
                variant: ButtonVariant::Unstyled,
                size: ButtonSize::IconLg,
                radius: ButtonRadius::Lg,
                align: ButtonAlign::Center,
                aria_label: Some("Sort".to_string()),
                title: Some("Sort".to_string()),
                aria_expanded: Some((if sort_popover() { "true" } else { "false" }).to_string()),
                class: "inline-flex items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground".to_string(),
                onclick: move |_| sort_popover.set(!sort_popover()),
                ArrowUpDown { size: 16 }
            }
            Popover {
                open: sort_popover(),
                on_close: move |_| sort_popover.set(false),
                align: PopoverAlign::CenterSm,
                class: "w-[min(13rem,calc(100vw-2rem))] space-y-2 p-3".to_string(),
                label { class: "text-xs font-medium text-muted-foreground", "Sort by" }
                SelectMenu {
                    value: sort_cfg().field,
                    options: vec![
                        SelectOption::new("date", "Date"),
                        SelectOption::new("amount", "Price"),
                        SelectOption::new("name", "Name"),
                        SelectOption::new("created_at", "Created"),
                        SelectOption::new("updated_at", "Last modified"),
                    ],
                    on_change: move |value: String| {
                        let mut next = sort_cfg();
                        next.field = value;
                        sort_cfg.set(next);
                    },
                    searchable: false,
                }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    full_width: true,
                    align: ButtonAlign::Between,
                    onclick: move |_| {
                        let mut next = sort_cfg();
                        next.direction = if next.direction == "desc" { "asc".into() } else { "desc".into() };
                        sort_cfg.set(next);
                    },
                    span { if sort_cfg().direction == "desc" { "Descending" } else { "Ascending" } }
                    span { if sort_cfg().direction == "desc" { "↓" } else { "↑" } }
                }
            }
        }
        Button {
            variant: ButtonVariant::Unstyled,
            size: ButtonSize::IconLg,
            radius: ButtonRadius::Lg,
            align: ButtonAlign::Center,
            aria_label: Some((if selection_mode() { "Exit selection mode" } else { "Enter selection mode" }).to_string()),
            aria_pressed: Some((if selection_mode() { "true" } else { "false" }).to_string()),
            class: (if selection_mode() { "inline-flex items-center justify-center rounded-lg border border-primary/40 bg-primary/10 text-primary" } else { "inline-flex items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground" }).to_string(),
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
