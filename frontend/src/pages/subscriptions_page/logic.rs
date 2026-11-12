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
    pub debts_only: bool,
    pub debt_subscription_ids: Vec<Uuid>,
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
    ("finalized", "Finalized"),
];

pub const SORT_OPTIONS: &[(&str, &str)] = &[
    ("next_billing_date", "Upcoming date"),
    ("amount", "Price"),
    ("name", "Name"),
    ("frequency", "Repeat"),
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
    if f.debts_only {
        n += 1;
    }
    n
}

/// Case-insensitive search over the fields a user can recognize at a glance.
pub fn matches_search(name: &str, notes: Option<&str>, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }
    name.to_lowercase().contains(&query)
        || notes
            .is_some_and(|notes| notes.to_lowercase().contains(&query))
}

pub fn filter_subscriptions_by_search(
    subs: Vec<Subscription>,
    query: &str,
) -> Vec<Subscription> {
    subs.into_iter()
        .filter(|sub| matches_search(&sub.name, sub.notes.as_deref(), query))
        .collect()
}

pub fn filter_subscriptions(subs: &[Subscription], filters: &FilterConfig) -> Vec<Subscription> {
    let mut result: Vec<Subscription> = subs.to_vec();

    match filters.status.as_deref() {
        Some("active") => result.retain(|s| s.is_active && !subscription_is_finalized(s.is_active, s.end_date, chrono::Utc::now().date_naive())),
        Some("inactive") => result.retain(|s| !s.is_active),
        Some("finalized") => result.retain(|s| subscription_is_finalized(s.is_active, s.end_date, chrono::Utc::now().date_naive())),
        _ => {}
    }

    if let Some(ref unit) = filters.frequency {
        result.retain(|s| s.recurrence.unit == *unit || s.frequency == *unit);
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

    if filters.debts_only {
        result.retain(|s| filters.debt_subscription_ids.contains(&s.id));
    }

    result
}

pub fn sort_subscriptions(subs: &mut [Subscription], sort: &SortConfig) {
    let unit_order: std::collections::HashMap<&str, i32> = std::collections::HashMap::from([
        ("day", 0),
        ("week", 1),
        ("month", 2),
        ("year", 3),
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
                let legacy_order: std::collections::HashMap<&str, i32> =
                    std::collections::HashMap::from([
                        ("daily", 0),
                        ("weekly", 1),
                        ("monthly", 2),
                        ("quarterly", 3),
                        ("yearly", 4),
                    ]);
                let a_ord = unit_order.get(a.recurrence.unit.as_str()).copied().unwrap_or(99);
                let b_ord = unit_order.get(b.recurrence.unit.as_str()).copied().unwrap_or(99);
                a_ord
                    .cmp(&b_ord)
                    .then_with(|| a.recurrence.interval.cmp(&b.recurrence.interval))
                    .then_with(|| {
                        let a_legacy = legacy_order.get(a.frequency.as_str()).copied().unwrap_or(99);
                        let b_legacy = legacy_order.get(b.frequency.as_str()).copied().unwrap_or(99);
                        a_legacy.cmp(&b_legacy)
                    })
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

/// A subscription is finalized after its inclusive end date has passed.
pub fn subscription_is_finalized(
    is_active: bool,
    end_date: Option<chrono::NaiveDate>,
    today: chrono::NaiveDate,
) -> bool {
    is_active && end_date.is_some_and(|end| end < today)
}

/// Returns CSS opacity class based on active/finalized status.
pub fn subscription_card_opacity(is_active: bool, finalized: bool) -> &'static str {
    if is_active && !finalized { "" } else { "opacity-60" }
}

/// Human-readable end-date state shown under the next payment line.
pub fn subscription_end_text(
    end_date: Option<chrono::NaiveDate>,
    today: chrono::NaiveDate,
    date_format: &str,
) -> Option<String> {
    let end = end_date?;
    if end < today {
        Some(format!("Finalizó el {}", format_date(&end, date_format)))
    } else {
        Some(format!("Finaliza el {}", format_date(&end, date_format)))
    }
}

/// Split subscriptions into active, inactive and finalized sections.
pub fn subscription_sections(
    subscriptions: &[Subscription],
    today: chrono::NaiveDate,
) -> (Vec<Subscription>, Vec<Subscription>, Vec<Subscription>) {
    let mut active = Vec::new();
    let mut inactive = Vec::new();
    let mut finalized = Vec::new();
    for sub in subscriptions {
        let is_finalized = subscription_is_finalized(sub.is_active, sub.end_date, today);
        if !sub.is_active {
            inactive.push(sub.clone());
        } else if is_finalized {
            finalized.push(sub.clone());
        } else {
            active.push(sub.clone());
        }
    }
    (active, inactive, finalized)
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
