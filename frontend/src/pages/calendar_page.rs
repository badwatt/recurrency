use dioxus::prelude::*;
use lucide_dioxus::{ChevronLeft, ChevronRight, Pencil, Trash2};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::storage::Storage;
use crate::components::page_header::PageHeader;
use crate::components::skeleton::SkeletonCalendar;
use crate::components::dialog::Dialog;
use crate::components::logo_image_with_fallback;
use crate::components::entry_form::{EntryForm as EditForm, validate_entry_form, EntityType};
use crate::models::subscription::{CalendarItem, CreateSubscriptionRequest, UpdateSubscriptionRequest};
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::preferences::PREFERENCES;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::utils::constants::{CURRENCIES, FREQUENCIES};
use crate::utils::format::format_currency;
use crate::utils::icons::icon_display;
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

const WEEKDAYS_SUNDAY: &[&str] = &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const WEEKDAYS_MONDAY: &[&str] = &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const MONTH_NAMES: &[&str] = &[
    "",
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Compute calendar grid days (with null padding for leading empty cells).
pub fn calendar_days(year: i32, month: u32, first_day_of_week: &str) -> Vec<Option<u32>> {
    let first_day = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    let start_padding = if first_day_of_week == "sunday" {
        first_day.weekday().num_days_from_sunday()
    } else {
        first_day.weekday().num_days_from_monday()
    };
    let total_days = days_in_month(year, month);
    let mut days: Vec<Option<u32>> = (0..start_padding).map(|_| None).collect();
    for d in 1..=total_days {
        days.push(Some(d));
    }
    days
}

/// Number of days in a given month.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    let default_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap_or(default_date))
        .pred_opt()
        .unwrap_or(default_date)
        .day()
}

/// Get items for a specific day.
pub fn items_for_day(
    items: &[CalendarItem],
    year: i32,
    month: u32,
    day: u32,
) -> Vec<&CalendarItem> {
    let date_str = format!("{}-{:02}-{:02}", year, month, day);
    items
        .iter()
        .filter(|i| i.billing_date.to_string() == date_str)
        .collect()
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
    logo_url: Option<String>,
}

fn item_display(item: &CalendarItem, currency: &str) -> CalendarItemDisplay {
    CalendarItemDisplay {
        subscription_id: item.subscription_id,
        name: item.name.clone(),
        amount_str: format_currency(item.amount, currency),
        title_str: format!(
            "{} - {}",
            item.name,
            format_currency(item.amount, &item.currency)
        ),
        logo_url: item.logo_url.clone(),
    }
}

/// Pre-computed display info for a day cell.
struct DayCellInfo {
    day: u32,
    is_today: bool,
    items: Vec<CalendarItemDisplay>,
    extra_count: usize,
}

fn day_cell_info(
    items: &[CalendarItem],
    year: i32,
    month: u32,
    day: u32,
    currency: &str,
) -> DayCellInfo {
    let day_items = items_for_day(items, year, month, day);
    let display_items: Vec<CalendarItemDisplay> = day_items
        .iter()
        .take(3)
        .map(|i| item_display(i, currency))
        .collect();
    DayCellInfo {
        day,
        is_today: is_today(year, month, day),
        items: display_items,
        extra_count: day_items.len().saturating_sub(3),
    }
}

// EditForm now uses shared SubscriptionForm (aliased above)

/// Helper: find category name for a CalendarItem by looking up the subscription in SUBSCRIPTIONS.
fn category_name_for_item(item: &CalendarItem) -> String {
    let subs = SUBSCRIPTIONS();
    if let Some(sub) = subs
        .subscriptions
        .iter()
        .find(|s| s.id == item.subscription_id)
    {
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
    if let Some(sub) = subs
        .subscriptions
        .iter()
        .find(|s| s.id == item.subscription_id)
    {
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

    /// Helper: find category icon for a CalendarItem.
fn category_icon_for_item(item: &CalendarItem) -> String {
    let subs = SUBSCRIPTIONS();
    if let Some(sub) = subs
        .subscriptions
        .iter()
        .find(|s| s.id == item.subscription_id)
    {
        let cats = CATEGORIES();
        if let Some(cat) = cats.get_by_id(&sub.category_id) {
            return cat.icon.clone().unwrap_or_else(|| "Tag".to_string());
        }
    }
    "Tag".to_string()
}

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
        let subs = SUBSCRIPTIONS();
        if let Some(sub) = subs.subscriptions.iter().find(|s| s.id == sub_id) {
            editing_id.set(Some(sub.id));
            edit_form.set(EditForm::from_entry(
                &sub.name,
                &sub.amount.to_string(),
                &sub.currency,
                &sub.frequency,
                &sub.start_date.to_string(),
                &sub.category_id.to_string(),
                &sub.notes.clone().unwrap_or_default(),
                &sub.logo_url.clone().unwrap_or_default(),
            ));
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
                let _ = Storage::delete_subscription(&id).await;
                match Storage::list_subscriptions().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
                // Also refresh calendar
                let y = current_year();
                let m = current_month();
                if let Ok(items) = Storage::calendar_month(y, m).await {
                    calendar_items.set(items)
                }
            });
            delete_confirm_id.set(None);
        }
    };

    // Handler: submit edit form
    let handle_edit_submit = move |_| {
        let f = edit_form();
        let result = validate_entry_form(
            &f.name, &f.amount, &f.currency, &f.frequency, &f.start_date, &f.category_id, EntityType::Subscription,
        );
        if !result.is_valid() {
            name_error.set(result.name_error.unwrap_or_default());
            amount_error.set(result.amount_error.unwrap_or_default());
            currency_error.set(result.currency_error.unwrap_or_default());
            frequency_error.set(result.frequency_error.unwrap_or_default());
            start_date_error.set(result.start_date_error.unwrap_or_default());
            category_error.set(result.category_error.unwrap_or_default());
            return;
        }

        let amount = f.amount.parse::<Decimal>().unwrap_or(dec!(0));
        let category = if f.category_id.is_empty() {
            None
        } else {
            Uuid::parse_str(&f.category_id).ok()
        };

        if let Some(id) = editing_id() {
            let req = UpdateSubscriptionRequest {
                name: Some(f.name),
                amount: Some(amount),
                currency: Some(f.currency),
                frequency: Some(f.frequency),
                start_date: Some(
                    f.start_date
                        .parse()
                        .unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                            .expect("2024-01-01 is a valid date")),
                ),
                category_id: category,
                notes: if f.notes.is_empty() {
                    None
                } else {
                    Some(f.notes.clone())
                },
                logo_url: if f.logo_url.is_empty() {
                    None
                } else {
                    Some(f.logo_url.clone())
                },
                ..Default::default()
            };
            spawn(async move {
                let _ = Storage::update_subscription(&id, &req).await;
                match Storage::list_subscriptions().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
                // Refresh calendar
                let y = current_year();
                let m = current_month();
                if let Ok(items) = Storage::calendar_month(y, m).await {
                    calendar_items.set(items)
                }
            });
        } else {
            let req = CreateSubscriptionRequest {
                name: f.name,
                amount,
                currency: f.currency,
                frequency: f.frequency,
                start_date: f.start_date.parse().unwrap_or(
                    chrono::NaiveDate::from_ymd_opt(2024, 1, 1).expect("2024-01-01 is a valid date"),
                ),
                category_id: category,
                notes: if f.notes.is_empty() {
                    None
                } else {
                    Some(f.notes.clone())
                },
                logo_url: if f.logo_url.is_empty() {
                    None
                } else {
                    Some(f.logo_url.clone())
                },
            };
            spawn(async move {
                let _ = Storage::create_subscription(&req).await;
                match Storage::list_subscriptions().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
                // Refresh calendar
                let y = current_year();
                let m = current_month();
                if let Ok(items) = Storage::calendar_month(y, m).await {
                    calendar_items.set(items)
                }
            });
        }

        edit_dialog_open.set(false);
    };

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
    let edit_title = if is_editing {
        "Edit Subscription".to_string()
    } else {
        "Add Subscription".to_string()
    };

    rsx! {
        div { class: "p-4 md:p-6 flex flex-col gap-6",
            // Header
            div { class: "flex flex-col sm:flex-row sm:items-end sm:justify-between gap-3 animate-in fade-in slide-in-from-bottom-4 duration-700",
                PageHeader { title: "Calendar", subtitle: "Upcoming subscription payments" }
                div { class: "text-right hidden md:block animate-in fade-in slide-in-from-bottom-4 duration-700 delay-150",
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
            div { class: "hidden md:block rounded-xl bg-card/80 border border-border/40 animate-in fade-in slide-in-from-bottom-4 duration-700 delay-200",
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
                                                            let default_cat = CATEGORIES()
                                                                .default_category()
                                                                .map(|c| c.id.to_string())
                                                                .unwrap_or_default();
                                                            edit_form.set(EditForm::init_for_create(
                                                                &formatted_date,
                                                                AUTH().currency(),
                                                                &default_cat,
                                                                EntityType::Subscription,
                                                            ));
                                                            editing_id.set(None);
                                                            name_error.set(String::new());
                                                            amount_error.set(String::new());
                                                            currency_error.set(String::new());
                                                            frequency_error.set(String::new());
                                                            start_date_error.set(String::new());
                                                            category_error.set(String::new());
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
                div {
                    class: "fixed inset-0 z-[60] flex items-center justify-center bg-black/50",
                    onclick: move |_| edit_dialog_open.set(false),
                    div {
                        class: "bg-background rounded-none sm:rounded-lg p-4 sm:p-6 w-full sm:max-w-[425px] shadow-lg sm:mx-4 h-dvh sm:h-auto sm:max-h-[85vh] flex flex-col",
                        onclick: move |e| e.stop_propagation(),
                        h2 { class: "text-lg font-semibold", {edit_title.to_string()} }
                        p { class: "text-sm text-muted-foreground mb-4", "Make changes to your subscription here." }

                        div { class: "space-y-4",
                            // Name
                            div { class: "space-y-1.5",
                                label { class: "text-sm font-medium", "Name" }
                                input {
                                    class: if name_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                                    placeholder: "Netflix, Spotify, etc.",
                                    value: form_name.to_string(),
                                    oninput: move |e: Event<FormData>| {
                                        edit_form.write().name = e.value();
                                        name_error.set(String::new());
                                    },
                                }
                                if !name_err.is_empty() {
                                    p { class: "text-xs text-red-500", {name_err.to_string()} }
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
                                        value: form_amount.to_string(),
                                        oninput: move |e: Event<FormData>| {
                                            edit_form.write().amount = e.value();
                                            amount_error.set(String::new());
                                        },
                                    }
                                    if !amount_err.is_empty() {
                                        p { class: "text-xs text-red-500", {amount_err.to_string()} }
                                    }
                                }
                                div { class: "space-y-1.5",
                                    label { class: "text-sm font-medium", "Currency" }
                                    select {
                                        class: if currency_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                                        value: form_currency.to_string(),
                                        onchange: move |e: Event<FormData>| {
                                            edit_form.write().currency = e.value();
                                            currency_error.set(String::new());
                                        },
                                        for (val, label) in CURRENCIES {
                                            option { value: val.to_string(), {label.to_string()} }
                                        }
                                    }
                                    if !currency_err.is_empty() {
                                        p { class: "text-xs text-red-500", {currency_err.to_string()} }
                                    }
                                }
                            }

                            // Frequency + Start date
                            div { class: "grid grid-cols-2 gap-4",
                                div { class: "space-y-1.5",
                                    label { class: "text-sm font-medium", "Frequency" }
                                    select {
                                        class: if frequency_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                                        value: form_frequency.to_string(),
                                        onchange: move |e: Event<FormData>| {
                                            edit_form.write().frequency = e.value();
                                            frequency_error.set(String::new());
                                        },
                                        for (val, label) in FREQUENCIES {
                                            option { value: val.to_string(), selected: form_frequency == *val, {label.to_string()} }
                                        }
                                    }
                                    if !frequency_err.is_empty() {
                                        p { class: "text-xs text-red-500", {frequency_err.to_string()} }
                                    }
                                }
                                div { class: "space-y-1.5",
                                    label { class: "text-sm font-medium", "Start Date" }
                                    input {
                                        r#type: "date",
                                        class: if start_date_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                                        value: form_start_date.to_string(),
                                        oninput: move |e: Event<FormData>| {
                                            edit_form.write().start_date = e.value();
                                            start_date_error.set(String::new());
                                        },
                                    }
                                    if !start_date_err.is_empty() {
                                        p { class: "text-xs text-red-500", {start_date_err.to_string()} }
                                    }
                                }
                            }

                            // Category
                            div { class: "space-y-1.5",
                                label { class: "text-sm font-medium", "Category" }
                                {rsx! {
                                    crate::components::CategoryCombobox {
                                        selected_id: form_category_id,
                                        on_change: move |id: String| {
                                            edit_form.write().category_id = id;
                                            category_error.set(String::new());
                                        },
                                        error: if category_err.is_empty() { None } else { Some(category_err) },
                                    }
                                }}
                            }

                            // Notes
                            div { class: "space-y-1.5",
                                label { class: "text-sm font-medium", "Notes (optional)" }
                                textarea {
                                    class: "w-full min-h-[80px] rounded-md border bg-background px-3 py-2 text-sm resize-none",
                                    placeholder: "Additional notes...",
                                    value: form_notes.to_string(),
                                    oninput: move |e: Event<FormData>| edit_form.write().notes = e.value(),
                                }
                            }
                        }

                        // Footer
                        div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2 mt-6",
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
                    div { class: "bg-background rounded-none sm:rounded-lg p-4 sm:p-6 w-full sm:w-80 space-y-4 shadow-lg h-dvh sm:h-auto flex flex-col",
                        h3 { class: "font-semibold", "Delete subscription?" }
                        p { class: "text-sm text-muted-foreground",
                            "This action cannot be undone."
                        }
                        div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
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
mod tests;
