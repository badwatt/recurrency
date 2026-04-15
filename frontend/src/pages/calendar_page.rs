use dioxus::prelude::*;
use crate::state::auth::AUTH;
use crate::api::subscriptions;
use crate::models::subscription::CalendarItem;
use crate::utils::format::format_currency;
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

#[component]
pub fn CalendarPage() -> Element {
    let mut current_year = use_signal(|| Local::now().year());
    let mut current_month = use_signal(|| Local::now().month());
    let mut calendar_items = use_signal(Vec::<CalendarItem>::new);
    let mut is_loading = use_signal(|| false);
    let mut selected_day = use_signal(|| None::<(i32, u32, u32)>);

    let year = current_year();
    let month = current_month();

    // Load calendar data
    use_future(move || async move {
        is_loading.set(true);
        match subscriptions::calendar_month(year, month).await {
            Ok(items) => calendar_items.set(items),
            Err(e) => log::error!("Failed to fetch calendar: {}", e),
        }
        is_loading.set(false);
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

    // Selected day dialog data
    let selected_dialog = selected_day().map(|(sy, sm, sd)| {
        let label = format!("{} {} {}", MONTH_NAMES.get(sm as usize).unwrap_or(&""), sd, sy);
        let day_item_refs = items_for_day(&items, sy, sm, sd);
        let day_displays: Vec<(uuid::Uuid, String, String)> = day_item_refs.iter().map(|item| {
            (item.subscription_id, item.name.clone(), format_currency(item.amount, &item.currency))
        }).collect();
        (label, day_displays)
    });

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

            // Day detail modal
            if let Some((label, day_displays)) = selected_dialog {
                div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
                    div { class: "w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg mx-4",
                        h2 { class: "text-lg font-semibold mb-4", "{label}" }
                        if day_displays.is_empty() {
                            p { class: "text-muted-foreground", "No payments scheduled" }
                        } else {
                            div { class: "space-y-2",
                                for (id, name, amount_str) in &day_displays {
                                    {rsx! {
                                        div { key: "{id}", class: "flex items-center justify-between p-3 rounded-lg border border-border",
                                            div { class: "flex items-center gap-2",
                                                {icon_credit_card()}
                                                span { class: "font-medium", "{name}" }
                                            }
                                            span { class: "font-bold", "{amount_str}" }
                                        }
                                    }}
                                }
                            }
                        }
                        div { class: "mt-4 flex justify-end",
                            button {
                                onclick: move |_| selected_day.set(None),
                                class: "h-10 px-4 py-2 rounded-md border border-border text-sm font-medium hover:bg-accent",
                                "Close"
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
    use uuid::Uuid;

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
}