//! Pure filtering/sorting logic and card display helpers for the
//! subscriptions page. Kept free of the Dioxus runtime so they can be tested
//! without mounting components.

use crate::models::subscription::Subscription;
use crate::utils::dates::format_date;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilterConfig {
    pub status: Option<String>,
    pub frequency: Option<String>,
    pub category_id: Option<String>,
    pub price_min: Option<String>,
    pub price_max: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortConfig {
    pub field: String,
    pub direction: String,
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            field: "next_billing_date".into(),
            direction: "asc".into(),
        }
    }
}

pub const STATUS_OPTIONS: &[(&str, &str)] = &[
    ("all", "All"),
    ("active", "Active"),
    ("inactive", "Inactive"),
];

pub const SORT_OPTIONS: &[(&str, &str)] = &[
    ("next_billing_date", "Upcoming date"),
    ("amount", "Price"),
    ("name", "Name"),
    ("frequency", "Frequency"),
    ("start_date", "Start date"),
];

pub fn count_active_filters(f: &FilterConfig) -> usize {
    let mut n = 0;
    if f.status.is_some() {
        n += 1;
    }
    if f.frequency.is_some() {
        n += 1;
    }
    if f.category_id.is_some() {
        n += 1;
    }
    if f.price_min.is_some() {
        n += 1;
    }
    if f.price_max.is_some() {
        n += 1;
    }
    n
}

pub fn filter_subscriptions(subs: &[Subscription], filters: &FilterConfig) -> Vec<Subscription> {
    let mut result: Vec<Subscription> = subs.to_vec();

    match filters.status.as_deref() {
        Some("active") => result.retain(|s| s.is_active),
        Some("inactive") => result.retain(|s| !s.is_active),
        _ => {}
    }

    if let Some(ref freq) = filters.frequency {
        result.retain(|s| s.frequency == *freq);
    }

    if let Some(ref cat_id) = filters.category_id
        && let Ok(uid) = Uuid::parse_str(cat_id)
    {
        result.retain(|s| s.category_id == uid);
    }

    if let Some(ref min_s) = filters.price_min
        && let Ok(min) = min_s.parse::<Decimal>()
    {
        result.retain(|s| s.amount >= min);
    }

    if let Some(ref max_s) = filters.price_max
        && let Ok(max) = max_s.parse::<Decimal>()
    {
        result.retain(|s| s.amount <= max);
    }

    result
}

pub fn sort_subscriptions(subs: &mut [Subscription], sort: &SortConfig) {
    let freq_order: std::collections::HashMap<&str, i32> = std::collections::HashMap::from([
        ("weekly", 0),
        ("monthly", 1),
        ("quarterly", 2),
        ("yearly", 3),
    ]);

    subs.sort_by(|a, b| {
        let cmp = match sort.field.as_str() {
            "next_billing_date" => {
                let a_val = a
                    .next_billing_date
                    .map(|d| d.and_time(chrono::NaiveTime::MIN).and_utc().timestamp_millis())
                    .unwrap_or(i64::MAX);
                let b_val = b
                    .next_billing_date
                    .map(|d| d.and_time(chrono::NaiveTime::MIN).and_utc().timestamp_millis())
                    .unwrap_or(i64::MAX);
                a_val.cmp(&b_val)
            }
            "amount" => a.amount.cmp(&b.amount),
            "name" => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            "frequency" => {
                let a_ord = freq_order.get(a.frequency.as_str()).copied().unwrap_or(99);
                let b_ord = freq_order.get(b.frequency.as_str()).copied().unwrap_or(99);
                a_ord.cmp(&b_ord)
            }
            "start_date" => {
                let a_val = a
                    .start_date
                    .and_time(chrono::NaiveTime::MIN)
                    .and_utc()
                    .timestamp_millis();
                let b_val = b
                    .start_date
                    .and_time(chrono::NaiveTime::MIN)
                    .and_utc()
                    .timestamp_millis();
                a_val.cmp(&b_val)
            }
            _ => std::cmp::Ordering::Equal,
        };
        if sort.direction == "desc" {
            cmp.reverse()
        } else {
            cmp
        }
    });
}

pub fn apply_filters_and_sort(
    subs: &[Subscription],
    filters: &FilterConfig,
    sort: &SortConfig,
) -> Vec<Subscription> {
    let mut result = filter_subscriptions(subs, filters);
    sort_subscriptions(&mut result, sort);
    result
}

/// Returns CSS opacity class based on active status.
pub fn subscription_card_opacity(is_active: bool) -> &'static str {
    if is_active { "" } else { "opacity-60" }
}

/// Formats next billing date text.
pub fn subscription_next_text(
    is_active: bool,
    next_billing_date: Option<chrono::NaiveDate>,
    date_format: &str,
) -> String {
    if is_active {
        next_billing_date
            .map(|d| format!("Next: {}", format_date(&d, date_format)))
            .unwrap_or_else(|| "No billing date".into())
    } else {
        "Inactive".into()
    }
}

/// Returns true if subscription has non-empty notes.
pub fn subscription_has_notes(notes: &Option<String>) -> bool {
    notes.as_ref().is_some_and(|n| !n.is_empty())
}
