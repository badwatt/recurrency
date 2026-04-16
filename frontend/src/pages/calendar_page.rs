use dioxus::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::state::auth::AUTH;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::state::categories::CATEGORIES;
use crate::api;
use crate::api::subscriptions as sub_api;
use crate::models::subscription::{CalendarItem, UpdateSubscriptionRequest};
use crate::utils::format::format_currency;
use crate::utils::constants::{CURRENCIES, FREQUENCIES};
use crate::components::dialog::Dialog;
use chrono::{Datelike, Local, NaiveDate};

fn icon_calendar() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "20", height: "20", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        rect { x: "3", y: "4", width: "18", height: "18", rx: "2", ry: "2" }
        line { x1: "16", y1: "2", x2: "16", y2: "6" }
        line { x1: "8", y1: "2", x2: "8", y2: "6" }
        line { x1: "3", y1: "10", x2: "21", y2: "10" }
    }}
}

fn icon_credit_card() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        rect { x: "1", y: "4", width: "22", height: "16", rx: "2" }
        line { x1: "1", y1: "10", x2: "23", y2: "10" }
    }}
}

const WEEKDAYS: &[&str] = &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTH_NAMES: &[&str] = &[
    "", "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

/// Compute calendar grid days (with null padding for leading empty cells).
pub fn calendar_days(year: i32, month: u32) -> Vec<Option<u32>> {
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let start_padding = first_day.weekday().num_days_from_sunday();
    let total_days = days_in_month(year, month);
    let mut days: Vec<Option<u32>> = (0..start_padding).map(|_| None).collect();
    for d in 1..=total_days {
        days.push(Some(d));
    }
    days
}

/// Number of days in a given month.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
        .pred_opt()
        .unwrap()
        .day()
}

/// Get items for a specific day.
pub fn items_for_day(items: &[CalendarItem], year: i32, month: u32, day: u32) -> Vec<&CalendarItem> {
    let date_str = format!("{}-{:02}-{:02}", year, month, day);
    items.iter().filter(|i| i.billing_date.to_string() == date_str).collect()
}

/// Check if a day is today.
pub fn is_today(year: i32, month: u32, day: u32) -> bool {
    let now = Local::now().date_naive();
    now.year() == year && now.month() == month && now.day() == day
}

/// Compute month total.
pub fn month_total(items: &[CalendarItem]) -> rust_decimal::Decimal {
    items.iter().map(|i| i.amount).sum()
}

/// Pre-computed display info for a calendar item.
struct CalendarItemDisplay {
    subscription_id: uuid::Uuid,
    name: String,
    amount_str: String,
    title_str: String,
}

fn item_display(item: &CalendarItem, currency: &str) -> CalendarItemDisplay {
    CalendarItemDisplay {
        subscription_id: item.subscription_id,
        name: item.name.clone(),
        amount_str: format_currency(item.amount, currency),
        title_str: format!("{} - {}", item.name, format_currency(item.amount, &item.currency)),
    }
}

/// Pre-computed display info for a day cell.
struct DayCellInfo {
    day: u32,
    is_today: bool,
    items: Vec<CalendarItemDisplay>,
    extra_count: usize,
}

fn day_cell_info(items: &[CalendarItem], year: i32, month: u32, day: u32, currency: &str) -> DayCellInfo {
    let day_items = items_for_day(items, year, month, day);
    let display_items: Vec<CalendarItemDisplay> = day_items.iter().take(3).map(|i| item_display(i, currency)).collect();
    DayCellInfo {
        day,
        is_today: is_today(year, month, day),
        items: display_items,
        extra_count: day_items.len().saturating_sub(3),
    }
}

/// Dialog form state for editing a subscription from the calendar modal.
#[derive(Debug, Clone, Default)]
struct EditForm {
    name: String,
    amount: String,
    currency: String,
    frequency: String,
    start_date: String,
    category_id: String,
    notes: String,
    logo_url: String,
}

/// Helper: find category name for a CalendarItem by looking up the subscription in SUBSCRIPTIONS.
fn category_name_for_item(item: &CalendarItem) -> String {
    let subs = SUBSCRIPTIONS();
    if let Some(sub) = subs.subscriptions.iter().find(|s| s.id == item.subscription_id) {
        let cats = CATEGORIES();
        if let Some(cat) = cats.get_by_id(&sub.category_id) {
            return cat.name.clone();
        }
    }
    "Uncategorized".to_string()
}

/// Helper: find category color for a CalendarItem.
fn category_color_for_item(item: &CalendarItem) -> String {
    let subs = SUBSCRIPTIONS();
    if let Some(sub) = subs.subscriptions.iter().find(|s| s.id == item.subscription_id) {
        let cats = CATEGORIES();
        if let Some(cat) = cats.get_by_id(&sub.category_id) {
            return cat.color.clone().unwrap_or_else(|| "#6b7280".to_string());
        }
    }
    "#6b7280".to_string()
}

/// Helper: format frequency display.
fn format_frequency(freq: &str) -> &str {
    match freq {
        "weekly" => "Weekly",
        "monthly" => "Monthly",
        "quarterly" => "Quarterly",
        "yearly" => "Yearly",
        _ => freq,
    }
}

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
    let mut edit_form = use_signal(EditForm::default);
    let mut name_error: Signal<String> = use_signal(String::new);
    let mut amount_error: Signal<String> = use_signal(String::new);
    let mut currency_error: Signal<String> = use_signal(String::new);
    let mut frequency_error: Signal<String> = use_signal(String::new);
    let mut start_date_error: Signal<String> = use_signal(String::new);
    let mut category_error: Signal<String> = use_signal(String::new);

    // Delete confirmation state
    let mut delete_confirm_id = use_signal(|| None::<Uuid>);

    let year = current_year();
    let month = current_month();

    // Load calendar data
    use_future(move || async move {
        is_loading.set(true);
        match sub_api::calendar_month(year, month).await {
            Ok(items) => calendar_items.set(items),
            Err(e) => log::error!("Failed to fetch calendar: {}", e),
        }
        is_loading.set(false);
    });

    // Ensure subscriptions are loaded for category lookups and edit actions
    use_effect(move || {
        spawn(async move {
            let state = SUBSCRIPTIONS();
            if !state.is_loading && state.subscriptions.is_empty() && state.error.is_none() {
                SUBSCRIPTIONS.write().set_loading();
                match api::subscriptions::list().await {
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
                match api::categories::list().await {
                    Ok(data) => CATEGORIES.write().set_data(data),
                    Err(e) => CATEGORIES.write().set_error(e.to_string()),
                }
            }
        });
    });

    let currency = AUTH().currency().to_string();
    let items = calendar_items();
    let total = month_total(&items);
    let total_display = format_currency(total, &currency);
    let days = calendar_days(year, month);
    let now = Local::now().date_naive();
    let month_label = format!("{} {}", MONTH_NAMES.get(month as usize).unwrap_or(&""), year);

    // Pre-compute day cells for desktop grid
    let day_cells: Vec<Option<DayCellInfo>> = days.iter().map(|d| {
        d.map(|d| day_cell_info(&items, year, month, d, &currency))
    }).collect();

    // Sort upcoming items by billing date for mobile view
    let mut upcoming: Vec<_> = items.iter().collect();
    upcoming.sort_by(|a, b| a.billing_date.cmp(&b.billing_date));
    let upcoming_display: Vec<(String, String, String, String)> = upcoming.iter().map(|item| {
        (
            item.subscription_id.to_string(),
            item.name.clone(),
            item.billing_date.format("%a, %b %e").to_string(),
            format_currency(item.amount, &item.currency),
        )
    }).collect();

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
    let selected_day_items: Vec<(CalendarItem, String, String)> = selected_day()
        .map(|(sy, sm, sd)| {
            let day_refs = items_for_day(&items, sy, sm, sd);
            day_refs.iter().map(|item| {
                let cat_name = category_name_for_item(item);
                let cat_color = category_color_for_item(item);
                ((*item).clone(), cat_name, cat_color)
            }).collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let selected_day_label = selected_day().map(|(sy, sm, sd)| {
        format!("{} {} {}", MONTH_NAMES.get(sm as usize).unwrap_or(&""), sd, sy)
    }).unwrap_or_default();

    // Handler: open edit dialog for a subscription
    let mut handle_edit = move |sub_id: Uuid| {
        let subs = SUBSCRIPTIONS();
        if let Some(sub) = subs.subscriptions.iter().find(|s| s.id == sub_id) {
            editing_id.set(Some(sub.id));
            edit_form.set(EditForm {
                name: sub.name.clone(),
                amount: sub.amount.to_string(),
                currency: sub.currency.clone(),
                frequency: sub.frequency.clone(),
                start_date: sub.start_date.to_string(),
                category_id: sub.category_id.to_string(),
                notes: sub.notes.clone().unwrap_or_default(),
                logo_url: sub.logo_url.clone().unwrap_or_default(),
            });
            name_error.set(String::new());
            amount_error.set(String::new());
            currency_error.set(String::new());
            frequency_error.set(String::new());
            start_date_error.set(String::new());
            category_error.set(String::new());
            edit_dialog_open.set(true);
        }
    };

    // Handler: open delete confirmation
    let mut handle_delete_request = move |sub_id: Uuid| {
        delete_confirm_id.set(Some(sub_id));
    };

    // Handler: confirm delete
    let handle_delete_confirm = move |_| {
        if let Some(id) = delete_confirm_id() {
            spawn(async move {
                let _ = api::subscriptions::delete(&id).await;
                match api::subscriptions::list().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
                // Also refresh calendar
                let y = current_year();
                let m = current_month();
                match sub_api::calendar_month(y, m).await {
                    Ok(items) => calendar_items.set(items),
                    Err(_) => {}
                }
            });
            delete_confirm_id.set(None);
        }
    };

    // Handler: submit edit form
    let handle_edit_submit = move |_| {
        let f = edit_form();
        let mut valid = true;

        if f.name.trim().is_empty() {
            name_error.set("Name is required".into());
            valid = false;
        }
        let amount_result = f.amount.parse::<Decimal>();
        if f.amount.is_empty() {
            amount_error.set("Amount is required".into());
            valid = false;
        } else if amount_result.is_err() || amount_result.as_ref().unwrap_or(&dec!(0)) <= &dec!(0) {
            amount_error.set("Amount must be greater than 0".into());
            valid = false;
        }
        if f.currency.is_empty() {
            currency_error.set("Currency is required".into());
            valid = false;
        }
        if f.frequency.is_empty() {
            frequency_error.set("Frequency is required".into());
            valid = false;
        }
        if f.start_date.is_empty() {
            start_date_error.set("Start date is required".into());
            valid = false;
        } else if f.start_date.parse::<chrono::NaiveDate>().is_err() {
            start_date_error.set("Invalid date format".into());
            valid = false;
        }
        if f.category_id.is_empty() {
            category_error.set("Category is required".into());
            valid = false;
        }

        if !valid { return; }

        let amount = f.amount.parse::<Decimal>().unwrap_or(dec!(0));
        let category = if f.category_id.is_empty() { None } else { Uuid::parse_str(&f.category_id).ok() };

        if let Some(id) = editing_id() {
            let req = UpdateSubscriptionRequest {
                name: Some(f.name),
                amount: Some(amount),
                currency: Some(f.currency),
                frequency: Some(f.frequency),
                start_date: Some(f.start_date.parse().unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())),
                category_id: category,
                notes: if f.notes.is_empty() { None } else { Some(f.notes.clone()) },
                logo_url: if f.logo_url.is_empty() { None } else { Some(f.logo_url.clone()) },
                ..Default::default()
            };
            spawn(async move {
                let _ = api::subscriptions::update(&id, &req).await;
                match api::subscriptions::list().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
                // Refresh calendar
                let y = current_year();
                let m = current_month();
                match sub_api::calendar_month(y, m).await {
                    Ok(items) => calendar_items.set(items),
                    Err(_) => {}
                }
            });
        }

        edit_dialog_open.set(false);
    };

    // Category options for form
    let category_options: Vec<(String, String)> = CATEGORIES()
        .categories.iter()
        .map(|c| (c.id.to_string(), c.name.clone()))
        .collect();

    let form_name = edit_form().name;
    let form_amount = edit_form().amount;
    let form_currency = edit_form().currency;
    let form_frequency = edit_form().frequency;
    let form_start_date = edit_form().start_date;
    let form_category_id = edit_form().category_id;
    let form_notes = edit_form().notes;

    let name_err = name_error();
    let amount_err = amount_error();
    let currency_err = currency_error();
    let frequency_err = frequency_error();
    let start_date_err = start_date_error();
    let category_err = category_error();

    let is_editing = editing_id().is_some();
    let edit_title = if is_editing { "Edit Subscription".to_string() } else { "Add Subscription".to_string() };

    rsx! {
        div { class: "p-4 md:p-6 space-y-6",
            // Header
            div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                div {
                    h1 { class: "text-3xl font-bold tracking-tight", "Calendar" }
                    p { class: "text-muted-foreground", "View your upcoming subscription payments" }
                }
                div { class: "text-right hidden md:block",
                    p { class: "text-2xl font-bold", "{total_display}" }
                    p { class: "text-sm text-muted-foreground", "Total this month" }
                }
            }

            // Mobile: upcoming payments list
            div { class: "md:hidden space-y-4",
                div { class: "flex items-center justify-between",
                    h2 { class: "text-lg font-semibold flex items-center gap-2",
                        {icon_calendar()}
                        "Upcoming Payments"
                    }
                    p { class: "text-lg font-bold", "{total_display}" }
                }

                {if is_loading() {
                    rsx! {
                        div { class: "flex h-48 items-center justify-center",
                            div { class: "h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" }
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
                            for (id, name, date_str, amount_str) in &upcoming_display {
                                {rsx! {
                                    div { key: "{id}", class: "rounded-lg border border-border bg-card p-3",
                                        div { class: "flex items-center justify-between",
                                            div { class: "flex items-center gap-3",
                                                div { class: "flex h-10 w-10 items-center justify-center rounded-full bg-primary/10",
                                                    span { class: "text-primary", {icon_credit_card()} }
                                                }
                                                div {
                                                    p { class: "font-medium", "{name}" }
                                                    p { class: "text-sm text-muted-foreground", "{date_str}" }
                                                }
                                            }
                                            p { class: "text-lg font-semibold", "{amount_str}" }
                                        }
                                    }
                                }}
                            }
                        }
                    }
                }}
            }

            // Desktop: Full calendar
            div { class: "hidden md:block rounded-lg border border-border bg-card",
                // Navigation header
                div { class: "flex items-center justify-between p-4 border-b border-border",
                    div { class: "flex items-center gap-4",
                        button {
                            onclick: go_prev,
                            class: "h-9 w-9 inline-flex items-center justify-center rounded-md border border-border hover:bg-accent",
                            "◀"
                        }
                        h2 { class: "text-xl font-semibold min-w-[200px] text-center", "{month_label}" }
                        button {
                            onclick: go_next,
                            class: "h-9 w-9 inline-flex items-center justify-center rounded-md border border-border hover:bg-accent",
                            "▶"
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
                        rsx! {
                            div { class: "flex h-96 items-center justify-center",
                                div { class: "h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" }
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "grid grid-cols-7 gap-px bg-border",
                                // Weekday headers
                                for day_name in WEEKDAYS {
                                    {rsx! {
                                        div { class: "bg-muted p-2 text-center text-sm font-medium text-muted-foreground",
                                            "{day_name}"
                                        }
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
                                                "min-h-[100px] bg-card p-2 hover:bg-muted/50 cursor-pointer ring-2 ring-primary"
                                            } else {
                                                "min-h-[100px] bg-card p-2 hover:bg-muted/50 cursor-pointer"
                                            };
                                            rsx! {
                                                div {
                                                    key: "{idx}",
                                                    class: "{cell_cls}",
                                                    onclick: move |_| selected_day.set(Some((year, month, d))),
                                                    // Day number
                                                    div { class: "{today_cls}", "{d}" }
                                                    // Items
                                                    div { class: "space-y-1",
                                                        for item in &info.items {
                                                            {rsx! {
                                                                div {
                                                                    key: "{item.subscription_id}",
                                                                    class: "flex items-center gap-1 rounded bg-primary/10 px-1 py-0.5 text-xs",
                                                                    title: "{item.title_str}",
                                                                    span { class: "shrink-0", {icon_credit_card()} }
                                                                    span { class: "truncate", "{item.name}" }
                                                                    span { class: "ml-auto shrink-0 font-medium", "{item.amount_str}" }
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
                            for (item, cat_name, cat_color) in &selected_day_items {
                                {rsx! {
                                    div {
                                        key: "{item.subscription_id}",
                                        class: "rounded-lg border border-border p-4 space-y-2",
                                        // Top row: name + amount
                                        div { class: "flex items-start justify-between gap-2",
                                            div { class: "flex-1 min-w-0",
                                                p { class: "font-semibold truncate", "{item.name}" }
                                                div { class: "flex items-center gap-2 mt-1 flex-wrap",
                                                    span {
                                                        class: "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-normal",
                                                        style: "color: {cat_color}; background-color: {cat_color}15;",
                                                        "{cat_name}"
                                                    }
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
                                                class: "text-xs px-3 py-1.5 rounded-md border border-border hover:bg-accent",
                                                onclick: {
                                                    let sub_id = item.subscription_id;
                                                    move |_| handle_edit(sub_id)
                                                },
                                                "✏ Edit"
                                            }
                                            button {
                                                class: "text-xs px-3 py-1.5 rounded-md border border-border text-red-600 hover:bg-red-50 dark:hover:bg-red-950",
                                                onclick: {
                                                    let sub_id = item.subscription_id;
                                                    move |_| handle_delete_request(sub_id)
                                                },
                                                "🗑 Delete"
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
                div { class: "fixed inset-0 z-[60] flex items-center justify-center bg-black/50",
                    div { class: "bg-background rounded-lg p-6 w-full max-w-[425px] shadow-lg mx-4",
                        h2 { class: "text-lg font-semibold", "{edit_title}" }
                        p { class: "text-sm text-muted-foreground mb-4", "Make changes to your subscription here." }

                        div { class: "space-y-4",
                            // Name
                            div { class: "space-y-1.5",
                                label { class: "text-sm font-medium", "Name" }
                                input {
                                    class: if name_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                                    placeholder: "Netflix, Spotify, etc.",
                                    value: "{form_name}",
                                    oninput: move |e: Event<FormData>| {
                                        edit_form.write().name = e.value();
                                        name_error.set(String::new());
                                    },
                                }
                                if !name_err.is_empty() {
                                    p { class: "text-xs text-red-500", "{name_err}" }
                                }
                            }

                            // Amount + Currency
                            div { class: "grid grid-cols-2 gap-4",
                                div { class: "space-y-1.5",
                                    label { class: "text-sm font-medium", "Amount" }
                                    input {
                                        r#type: "number",
                                        step: "0.01",
                                        min: "0",
                                        class: if amount_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                                        placeholder: "9.99",
                                        value: "{form_amount}",
                                        oninput: move |e: Event<FormData>| {
                                            edit_form.write().amount = e.value();
                                            amount_error.set(String::new());
                                        },
                                    }
                                    if !amount_err.is_empty() {
                                        p { class: "text-xs text-red-500", "{amount_err}" }
                                    }
                                }
                                div { class: "space-y-1.5",
                                    label { class: "text-sm font-medium", "Currency" }
                                    select {
                                        class: if currency_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                                        value: "{form_currency}",
                                        onchange: move |e: Event<FormData>| {
                                            edit_form.write().currency = e.value();
                                            currency_error.set(String::new());
                                        },
                                        for (val, label) in CURRENCIES {
                                            option { value: "{val}", "{label}" }
                                        }
                                    }
                                    if !currency_err.is_empty() {
                                        p { class: "text-xs text-red-500", "{currency_err}" }
                                    }
                                }
                            }

                            // Frequency + Start date
                            div { class: "grid grid-cols-2 gap-4",
                                div { class: "space-y-1.5",
                                    label { class: "text-sm font-medium", "Frequency" }
                                    select {
                                        class: if frequency_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                                        value: "{form_frequency}",
                                        onchange: move |e: Event<FormData>| {
                                            edit_form.write().frequency = e.value();
                                            frequency_error.set(String::new());
                                        },
                                        for (val, label) in FREQUENCIES {
                                            option { value: "{val}", "{label}" }
                                        }
                                    }
                                    if !frequency_err.is_empty() {
                                        p { class: "text-xs text-red-500", "{frequency_err}" }
                                    }
                                }
                                div { class: "space-y-1.5",
                                    label { class: "text-sm font-medium", "Start Date" }
                                    input {
                                        r#type: "date",
                                        class: if start_date_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                                        value: "{form_start_date}",
                                        oninput: move |e: Event<FormData>| {
                                            edit_form.write().start_date = e.value();
                                            start_date_error.set(String::new());
                                        },
                                    }
                                    if !start_date_err.is_empty() {
                                        p { class: "text-xs text-red-500", "{start_date_err}" }
                                    }
                                }
                            }

                            // Category
                            div { class: "space-y-1.5",
                                label { class: "text-sm font-medium", "Category" }
                                select {
                                    class: if category_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                                    value: "{form_category_id}",
                                    onchange: move |e: Event<FormData>| {
                                        edit_form.write().category_id = e.value();
                                        category_error.set(String::new());
                                    },
                                    option { value: "", "Select category..." }
                                    for (id, name) in category_options.iter() {
                                        option { value: "{id}", "{name}" }
                                    }
                                }
                                if !category_err.is_empty() {
                                    p { class: "text-xs text-red-500", "{category_err}" }
                                }
                            }

                            // Notes
                            div { class: "space-y-1.5",
                                label { class: "text-sm font-medium", "Notes (optional)" }
                                textarea {
                                    class: "w-full min-h-[80px] rounded-md border bg-background px-3 py-2 text-sm resize-none",
                                    placeholder: "Additional notes...",
                                    value: "{form_notes}",
                                    oninput: move |e: Event<FormData>| edit_form.write().notes = e.value(),
                                }
                            }
                        }

                        // Footer
                        div { class: "flex justify-end gap-2 mt-6",
                            button {
                                class: "border px-4 py-2 rounded-md text-sm hover:bg-muted",
                                onclick: move |_| edit_dialog_open.set(false),
                                "Cancel"
                            }
                            button {
                                class: "bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm hover:bg-primary/90",
                                onclick: handle_edit_submit,
                                "Save Changes"
                            }
                        }
                    }
                }
            }

            // Delete confirmation dialog
            if delete_confirm_id().is_some() {
                div { class: "fixed inset-0 z-[70] flex items-center justify-center bg-black/50",
                    div { class: "bg-background rounded-lg p-6 w-80 space-y-4 shadow-lg",
                        h3 { class: "font-semibold", "Delete subscription?" }
                        p { class: "text-sm text-muted-foreground",
                            "This action cannot be undone."
                        }
                        div { class: "flex justify-end gap-2",
                            button {
                                class: "border px-3 py-1.5 rounded-md text-sm hover:bg-muted",
                                onclick: move |_| delete_confirm_id.set(None),
                                "Cancel"
                            }
                            button {
                                class: "bg-red-600 text-white px-3 py-1.5 rounded-md text-sm hover:bg-red-700",
                                onclick: handle_delete_confirm,
                                "Delete"
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
    use rust_decimal_macros::dec;

    fn make_calendar_item(name: &str, amount: rust_decimal::Decimal, billing_date: NaiveDate) -> CalendarItem {
        CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: name.into(),
            amount,
            currency: "EUR".into(),
            frequency: "monthly".into(),
            billing_date,
            is_active: true,
            logo_url: None,
        }
    }

    #[test]
    fn calendar_days_padding() {
        let days = calendar_days(2024, 1);
        assert_eq!(days[0], None);
        assert_eq!(days[1], Some(1));
        assert_eq!(days[31], Some(31));
        assert_eq!(days.len(), 32);
    }

    #[test]
    fn calendar_days_february_leap() {
        let days = calendar_days(2024, 2);
        let day_count = days.iter().filter(|d| d.is_some()).count();
        assert_eq!(day_count, 29);
    }

    #[test]
    fn calendar_days_february_normal() {
        let days = calendar_days(2023, 2);
        let day_count = days.iter().filter(|d| d.is_some()).count();
        assert_eq!(day_count, 28);
    }

    #[test]
    fn days_in_month_various() {
        assert_eq!(days_in_month(2024, 1), 31);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2024, 4), 30);
        assert_eq!(days_in_month(2024, 12), 31);
    }

    #[test]
    fn items_for_day_match() {
        let items = vec![
            make_calendar_item("Netflix", dec!(15), NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            make_calendar_item("Spotify", dec!(10), NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            make_calendar_item("Hulu", dec!(12), NaiveDate::from_ymd_opt(2024, 1, 20).unwrap()),
        ];
        let day15 = items_for_day(&items, 2024, 1, 15);
        assert_eq!(day15.len(), 2);
        let day20 = items_for_day(&items, 2024, 1, 20);
        assert_eq!(day20.len(), 1);
        let day1 = items_for_day(&items, 2024, 1, 1);
        assert!(day1.is_empty());
    }

    #[test]
    fn month_total_sum() {
        let items = vec![
            make_calendar_item("A", dec!(10), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            make_calendar_item("B", dec!(20), NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
        ];
        assert_eq!(month_total(&items), dec!(30));
    }

    #[test]
    fn is_today_check() {
        let now = Local::now().date_naive();
        assert!(is_today(now.year(), now.month(), now.day()));
        assert!(!is_today(now.year(), now.month(), now.day() + 100));
    }

    #[test]
    fn day_cell_info_with_items() {
        let items = vec![
            make_calendar_item("A", dec!(10), NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
        ];
        let info = day_cell_info(&items, 2024, 1, 15, "EUR");
        assert_eq!(info.day, 15);
        assert_eq!(info.items.len(), 1);
        assert_eq!(info.extra_count, 0);
    }

    #[test]
    fn day_cell_info_extra_count() {
        let items = vec![
            make_calendar_item("Item0", rust_decimal::Decimal::from(1), NaiveDate::from_ymd_opt(2024, 3, 10).unwrap()),
            make_calendar_item("Item1", rust_decimal::Decimal::from(2), NaiveDate::from_ymd_opt(2024, 3, 10).unwrap()),
            make_calendar_item("Item2", rust_decimal::Decimal::from(3), NaiveDate::from_ymd_opt(2024, 3, 10).unwrap()),
            make_calendar_item("Item3", rust_decimal::Decimal::from(4), NaiveDate::from_ymd_opt(2024, 3, 10).unwrap()),
            make_calendar_item("Item4", rust_decimal::Decimal::from(5), NaiveDate::from_ymd_opt(2024, 3, 10).unwrap()),
        ];
        let info = day_cell_info(&items, 2024, 3, 10, "EUR");
        assert_eq!(info.items.len(), 3); // take(3)
        assert_eq!(info.extra_count, 2); // 5 - 3
    }

    #[test]
    fn format_frequency_test() {
        assert_eq!(format_frequency("weekly"), "Weekly");
        assert_eq!(format_frequency("monthly"), "Monthly");
        assert_eq!(format_frequency("quarterly"), "Quarterly");
        assert_eq!(format_frequency("yearly"), "Yearly");
        assert_eq!(format_frequency("custom"), "custom");
    }
}