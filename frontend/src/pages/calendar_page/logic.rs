//! Pure calendar logic and small presentational helpers, kept separate from
//! the page component so they can be unit-tested without the Dioxus runtime.

use crate::models::subscription::CalendarItem;
use crate::state::accounts::ACCOUNTS;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::utils::format::format_currency;
use chrono::{Datelike, Local, NaiveDate};
use dioxus::prelude::*;

pub fn icon_calendar() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "20", height: "20", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        rect { x: "3", y: "4", width: "18", height: "18", rx: "2", ry: "2" }
        line { x1: "16", y1: "2", x2: "16", y2: "6" }
        line { x1: "8", y1: "2", x2: "8", y2: "6" }
        line { x1: "3", y1: "10", x2: "21", y2: "10" }
    }}
}

pub fn icon_credit_card() -> Element {
    rsx! { svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
        rect { x: "1", y: "4", width: "22", height: "16", rx: "2" }
        line { x1: "1", y1: "10", x2: "23", y2: "10" }
    }}
}

pub const WEEKDAYS_SUNDAY: &[&str] = &["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
pub const WEEKDAYS_MONDAY: &[&str] = &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
pub const MONTH_NAMES: &[&str] = &[
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
pub struct CalendarItemDisplay {
    pub subscription_id: uuid::Uuid,
    pub name: String,
    pub amount_str: String,
    pub title_str: String,
    pub logo_url: Option<String>,
}

pub fn item_display(item: &CalendarItem, currency: &str) -> CalendarItemDisplay {
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
pub struct DayCellInfo {
    pub day: u32,
    pub is_today: bool,
    pub items: Vec<CalendarItemDisplay>,
    pub extra_count: usize,
}

/// An empty day cell opens the "add subscription" dialog; a day with payments
/// opens the day detail modal instead.
pub fn day_cell_is_empty(info: &DayCellInfo) -> bool {
    info.items.is_empty() && info.extra_count == 0
}

pub fn day_cell_info(
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

/// Helper: find category name for a CalendarItem by looking up the subscription in SUBSCRIPTIONS.
pub fn category_name_for_item(item: &CalendarItem) -> String {
    let subs = SUBSCRIPTIONS();
    if let Some(sub) = subs
        .subscriptions
        .iter()
        .find(|s| s.id == item.subscription_id)
    {
        let accounts = ACCOUNTS();
        if let Some(account) = accounts.get_by_id(&sub.account_id) {
            return account.name.clone();
        }
    }
    "No account".to_string()
}

/// Helper: find category color for a CalendarItem.
pub fn category_color_for_item(item: &CalendarItem) -> String {
    let subs = SUBSCRIPTIONS();
    if let Some(sub) = subs
        .subscriptions
        .iter()
        .find(|s| s.id == item.subscription_id)
    {
        let accounts = ACCOUNTS();
        if let Some(account) = accounts.get_by_id(&sub.account_id) {
            return account.color.clone().unwrap_or_else(|| "#6b7280".to_string());
        }
    }
    "#6b7280".to_string()
}

/// Helper: find category icon for a CalendarItem.
pub fn category_icon_for_item(item: &CalendarItem) -> String {
    let subs = SUBSCRIPTIONS();
    if let Some(sub) = subs
        .subscriptions
        .iter()
        .find(|s| s.id == item.subscription_id)
    {
        let accounts = ACCOUNTS();
        if let Some(account) = accounts.get_by_id(&sub.account_id) {
            return account.icon.clone().unwrap_or_else(|| "Tag".to_string());
        }
    }
    "Tag".to_string()
}

/// Helper: format frequency display.
pub fn format_frequency(freq: &str) -> &str {
    match freq {
        "weekly" => "Weekly",
        "monthly" => "Monthly",
        "quarterly" => "Quarterly",
        "yearly" => "Yearly",
        _ => freq,
    }
}
