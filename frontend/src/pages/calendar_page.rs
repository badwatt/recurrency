use dioxus::prelude::*;
use lucide_dioxus::{ChevronLeft, ChevronRight, Pencil, Trash2};
use uuid::Uuid;

use crate::storage::Storage;
use crate::components::page_header::PageHeader;
use crate::components::skeleton::SkeletonCalendar;
use crate::components::dialog::Dialog;
use crate::components::logo_image_with_fallback;
use crate::models::subscription::CalendarItem;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::preferences::PREFERENCES;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::utils::format::format_currency;
use crate::utils::icons::icon_display;
use chrono::{Datelike, Local};

mod delete_dialog;
mod edit_dialog;
mod logic;
pub use logic::*;
use delete_dialog::DeleteSubscriptionDialog;
use edit_dialog::EditSubscriptionDialog;

#[component]
pub fn CalendarPage() -> Element {
    let mut current_year = use_signal(|| Local::now().year());
    let mut current_month = use_signal(|| Local::now().month());
    let mut calendar_items = use_signal(Vec::<CalendarItem>::new);
    let mut is_loading = use_signal(|| false);
    let mut selected_day = use_signal(|| None::<(i32, u32, u32)>);

    // Edit dialog state
    let mut edit_dialog_open = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<Uuid>);
    let mut prefill_date = use_signal(|| None::<String>);

    // Delete confirmation state
    let mut delete_confirm_id = use_signal(|| None::<Uuid>);

    let year = current_year();
    let month = current_month();

    // Load calendar data — re-runs when year or month changes
    use_effect(move || {
        let year = current_year();
        let month = current_month();
        spawn(async move {
            is_loading.set(true);
            match Storage::calendar_month(year, month).await {
                Ok(items) => calendar_items.set(items),
                Err(e) => log::error!("Failed to fetch calendar: {}", e),
            }
            is_loading.set(false);
        });
    });

    // Ensure subscriptions are loaded for category lookups and edit actions
    use_effect(move || {
        spawn(async move {
            let state = SUBSCRIPTIONS();
            if !state.is_loading && state.subscriptions.is_empty() && state.error.is_none() {
                SUBSCRIPTIONS.write().set_loading();
                match Storage::list_subscriptions().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
            }
        });
    });

    // Ensure categories are loaded for category lookups
    use_effect(move || {
        spawn(async move {
            let state = CATEGORIES();
            if !state.is_loading && state.categories.is_empty() && state.error.is_none() {
                CATEGORIES.write().set_loading();
                match Storage::list_categories().await {
                    Ok(data) => CATEGORIES.write().set_data(data),
                    Err(e) => CATEGORIES.write().set_error(e.to_string()),
                }
            }
        });
    });

    let currency = AUTH().currency().to_string();
    let first_day = PREFERENCES().first_day_of_week().to_string();
    let items = calendar_items();
    let total = month_total(&items);
    let total_display = format_currency(total, &currency);
    let days = calendar_days(year, month, &first_day);
    let now = Local::now().date_naive();
    let month_label = format!(
        "{} {}",
        MONTH_NAMES.get(month as usize).unwrap_or(&""),
        year
    );
    let weekday_headers = if first_day == "sunday" {
        WEEKDAYS_SUNDAY
    } else {
        WEEKDAYS_MONDAY
    };

    // Pre-compute day cells for desktop grid
    let day_cells: Vec<Option<DayCellInfo>> = days
        .iter()
        .map(|d| d.map(|d| day_cell_info(&items, year, month, d, &currency)))
        .collect();

    // Sort upcoming items by billing date for mobile view
    let mut upcoming: Vec<_> = items.iter().collect();
    upcoming.sort_by_key(|a| a.billing_date);
    let upcoming_display: Vec<(String, String, String, String, Option<String>)> = upcoming
        .iter()
        .map(|item| {
            (
                item.subscription_id.to_string(),
                item.name.clone(),
                item.billing_date.format("%a, %b %e").to_string(),
                format_currency(item.amount, &item.currency),
                item.logo_url.clone(),
            )
        })
        .collect();

    let go_prev = move |_| {
        let (y, m) = if current_month() == 1 {
            (current_year() - 1, 12)
        } else {
            (current_year(), current_month() - 1)
        };
        current_year.set(y);
        current_month.set(m);
    };

    let go_next = move |_| {
        let (y, m) = if current_month() == 12 {
            (current_year() + 1, 1)
        } else {
            (current_year(), current_month() + 1)
        };
        current_year.set(y);
        current_month.set(m);
    };

    let go_today = move |_| {
        current_year.set(now.year());
        current_month.set(now.month());
    };

// Selected day dialog data: gather CalendarItems with full display info
    let selected_day_items: Vec<(CalendarItem, String, String, String)> = selected_day()
        .map(|(sy, sm, sd)| {
            let day_refs = items_for_day(&items, sy, sm, sd);
            day_refs
                .iter()
                .map(|item| {
                    let cat_name = category_name_for_item(item);
                    let cat_icon = category_icon_for_item(item);
                    let cat_color = category_color_for_item(item);
                    ((*item).clone(), cat_name, cat_icon, cat_color)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let selected_day_label = selected_day()
        .map(|(sy, sm, sd)| {
            format!(
                "{} {} {}",
                MONTH_NAMES.get(sm as usize).unwrap_or(&""),
                sd,
                sy
            )
        })
        .unwrap_or_default();

    // Handler: open edit dialog for a subscription
    let mut handle_edit = move |sub_id: Uuid| {
        editing_id.set(Some(sub_id));
        prefill_date.set(None);
        edit_dialog_open.set(true);
    };

    // Handler: open delete confirmation
    let mut handle_delete_request = move |sub_id: Uuid| {
        delete_confirm_id.set(Some(sub_id));
    };

    rsx! {
        div { class: "p-4 md:p-6 flex flex-col gap-6 animate-in fade-in duration-300",
            // Header
            div { class: "flex flex-col sm:flex-row sm:items-end sm:justify-between gap-3",
                PageHeader { title: "Calendar", subtitle: "Upcoming subscription payments" }
                div { class: "text-right hidden md:block",
                    p { class: "text-2xl font-bold tracking-tight", {total_display.to_string()} }
                    p { class: "text-xs text-muted-foreground", "Total this month" }
                }
            }

            // Mobile: upcoming payments list
            div { class: "md:hidden space-y-4",
                div { class: "flex items-center justify-between",
                    h2 { class: "text-lg font-semibold flex items-center gap-2",
                        {icon_calendar()}
                        "Upcoming Payments"
                    }
                    p { class: "text-lg font-bold", {total_display.to_string()} }
                }

                {if is_loading() {
                    rsx! {
                        div { class: "space-y-2",
                            div { class: "animate-pulse rounded-md bg-muted h-4 w-1/3" }
                            div { class: "animate-pulse rounded-md bg-muted h-3 w-2/3" }
                        }
                    }
                } else if upcoming_display.is_empty() {
                    rsx! {
                        div { class: "rounded-lg border border-border bg-card py-8 text-center text-muted-foreground",
                            "No payments scheduled this month"
                        }
                    }
                } else {
                    rsx! {
                        div { class: "space-y-2",
                            for (id, name, date_str, amount_str, logo_url) in &upcoming_display {
                                {rsx! {
                                    div { key: "{id}", class: "rounded-lg border border-border bg-card p-3",
                                        div { class: "flex items-center justify-between",
                                            div { class: "flex items-center gap-3",
                                                div { class: "flex h-10 w-10 items-center justify-center rounded-full bg-primary/10",
                                                    if let Some(url) = logo_url {
                                                        {logo_image_with_fallback(&format!("upcoming-{}", id), Some(url.as_str()), name, "h-6 w-6 rounded object-contain")
                                                        }
                                                    } else {
                                                        span { class: "text-primary", {icon_credit_card()} }
                                                    }
                                                }
                                                div {
                                                    p { class: "font-medium", {name.to_string()} }
                                                    p { class: "text-sm text-muted-foreground", {date_str.to_string()} }
                                                }
                                            }
                                            p { class: "text-lg font-semibold", {amount_str.to_string()} }
                                        }
                                    }
                                }}
                            }
                        }
                    }
                }}
            }

            // Desktop: Full calendar
            div { class: "hidden md:block rounded-xl bg-card/80 border border-border/40",
                // Navigation header
                div { class: "flex items-center justify-between p-4 border-b border-border",
                    div { class: "flex items-center gap-4",
                        button {
                            onclick: go_prev,
                            class: "h-9 w-9 inline-flex items-center justify-center rounded-md border border-border hover:bg-accent",
                            ChevronLeft { size: 16 }
                        }
                        h2 { class: "text-xl font-semibold min-w-[200px] text-center", {month_label.to_string()} }
                        button {
                            onclick: go_next,
                            class: "h-9 w-9 inline-flex items-center justify-center rounded-md border border-border hover:bg-accent",
                            ChevronRight { size: 16 }
                        }
                    }
                    button {
                        onclick: go_today,
                        class: "h-9 px-3 rounded-md border border-border text-sm hover:bg-accent",
                        "Today"
                    }
                }

                // Calendar grid
                div { class: "p-4",
                    {if is_loading() {
                        rsx! { SkeletonCalendar {} }
                    } else {
                        rsx! {
                            div { class: "grid grid-cols-7 gap-px bg-border",
                                // Weekday headers
                                for day_name in weekday_headers {
                                    {rsx! {
                                        div { class: "bg-muted p-2 text-center text-sm font-medium text-muted-foreground",
                                            {day_name.to_string()}                                        }
                                    }}
                                }
                                // Day cells
                                for (idx, cell) in day_cells.iter().enumerate() {
                                    {match cell {
                                        None => rsx! {
                                            div { key: "{idx}", class: "min-h-[100px] bg-muted/30 cursor-default p-2" }
                                        },
                                        Some(info) => {
                                            let d = info.day;
                                            let today_cls = if info.is_today {
                                                "mb-1 flex h-7 w-7 items-center justify-center rounded-full text-sm bg-primary text-primary-foreground font-bold"
                                            } else {
                                                "mb-1 flex h-7 w-7 items-center justify-center rounded-full text-sm"
                                            };
                                            let cell_cls = if info.is_today {
                                                "relative group min-h-[100px] bg-card p-2 hover:bg-muted/50 cursor-pointer border border-primary"
                                            } else {
                                                "relative group min-h-[100px] bg-card p-2 hover:bg-muted/50 cursor-pointer"
                                            };
                                            // Format date for pre-filling the subscription form
                                            let formatted_date = format!("{:04}-{:02}-{:02}", year, month, d);
                                            rsx! {
                                                div {
                                                    key: "{idx}",
                                                    class: cell_cls.to_string(),
                                                    onclick: move |_| selected_day.set(Some((year, month, d))),
                                                    // Day number
                                                    div { class: today_cls.to_string(), {d.to_string()} }
                                                    // Add subscription button (top-right, visible on hover)
                                                    button {
                                                        r#type: "button",
                                                        class: "absolute top-1 right-1 h-6 w-6 hidden group-hover:flex group-hover:opacity-100 items-center justify-center rounded-md bg-primary text-primary-foreground opacity-0 hover:bg-primary/80 transition-opacity",
                                                        title: "Add subscription for {formatted_date}",
                                                        onclick: move |e| {
                                                            e.stop_propagation();
                                                            prefill_date.set(Some(formatted_date.clone()));
                                                            editing_id.set(None);
                                                            edit_dialog_open.set(true);
                                                        },
                                                        "+"
                                                    }
                                                    // Items
                                                    div { class: "space-y-1",
                                                        for item in &info.items {
                                                            {rsx! {
                                                                div {
                                                                    key: "{item.subscription_id}",
                                                                    class: "flex items-center gap-1 rounded bg-primary/10 px-1 py-0.5 text-xs",
                                                                    title: item.title_str.to_string(),
                                                                    if let Some(url) = &item.logo_url {
                                                                        {logo_image_with_fallback(&format!("cal-{}", item.subscription_id), Some(url.as_str()), &item.name, "h-4 w-4 rounded object-contain")
                                                                        }
                                                                    } else {
                                                                        span { class: "shrink-0", {icon_credit_card()} }
                                                                    }
                                                                    span { class: "truncate", {item.name.to_string()} }
                                                                    span { class: "ml-auto shrink-0 font-medium", {item.amount_str.to_string()} }
                                                                }
                                                            }}
                                                        }
                                                        if info.extra_count > 0 {
                                                            div { class: "text-xs text-muted-foreground",
                                                                "+{info.extra_count} more"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }}
                                }
                            }
                        }
                    }}

                    // Legend
                    div { class: "mt-4 flex items-center gap-4 text-sm text-muted-foreground",
                        div { class: "flex items-center gap-2",
                            div { class: "h-4 w-4 rounded-full bg-primary" }
                            span { "Today" }
                        }
                        div { class: "flex items-center gap-2",
                            div { class: "h-4 w-4 rounded bg-primary/10" }
                            span { "Payment due" }
                        }
                    }
                }
            }

            // Day detail modal using Dialog component
            Dialog {
                open: selected_day().is_some(),
                on_close: move |_| selected_day.set(None),
                title: selected_day_label.clone(),
                {rsx! {
                    if selected_day_items.is_empty() {
                        p { class: "text-muted-foreground text-center py-4", "No payments scheduled" }
                    } else {
                        div { class: "space-y-3",
                            for (item, cat_name, cat_icon, cat_color) in &selected_day_items {
                                {rsx! {
                                    div {
                                        key: "{item.subscription_id}",
                                        class: "rounded-lg border border-border p-4 space-y-2",
                                        // Top row: name + amount
                                        div { class: "flex items-start justify-between gap-2",
                                            div { class: "flex-1 min-w-0",
                                                p { class: "font-semibold truncate", {item.name.to_string()} }
                                                div { class: "flex items-center gap-2 mt-1 flex-wrap",
                                                    span {
                                                        class: "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-normal",
                                                        style: "color: {cat_color}; background-color: {cat_color}15;",
                                                        {icon_display(cat_icon)}
                                                        {cat_name.to_string()}                                                    }
                                                    span { class: "text-xs text-muted-foreground",
                                                        "{format_frequency(&item.frequency)}"
                                                    }
                                                }
                                            }
                                            p { class: "text-lg font-bold shrink-0",
                                                "{format_currency(item.amount, &item.currency)}"
                                            }
                                        }
                                        // Action buttons
                                        div { class: "flex items-center gap-2 pt-1",
                                            button {
                                                class: "text-xs px-3 py-1.5 rounded-md border border-border hover:bg-accent inline-flex items-center gap-1",
                                                onclick: {
                                                    let sub_id = item.subscription_id;
                                                    move |_| handle_edit(sub_id)
                                                },
                                                Pencil { size: 14 }
                                                "Edit"
                                            }
                                            button {
                                                class: "text-xs px-3 py-1.5 rounded-md border border-border text-red-600 hover:bg-red-50 dark:hover:bg-red-950 inline-flex items-center gap-1",
                                                onclick: {
                                                    let sub_id = item.subscription_id;
                                                    move |_| handle_delete_request(sub_id)
                                                },
                                                Trash2 { size: 14 }
                                                "Delete"
                                            }
                                        }
                                    }
                                }}
                            }
                        }
                    }
                }}
            }

            // Edit subscription dialog
            if edit_dialog_open() {
                EditSubscriptionDialog {
                    open: edit_dialog_open,
                    editing_id: editing_id,
                    prefill_date: prefill_date,
                    calendar_items: calendar_items,
                    current_year: current_year,
                    current_month: current_month,
                }
            }

            // Delete confirmation dialog
            if delete_confirm_id().is_some() {
                DeleteSubscriptionDialog {
                    confirm_id: delete_confirm_id,
                    calendar_items: calendar_items,
                    current_year: current_year,
                    current_month: current_month,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
