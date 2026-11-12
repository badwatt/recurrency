use chrono::{Datelike, Local, NaiveDate};
use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use crate::components::ConvertedAmount;
use crate::components::logo_image_with_fallback;
use crate::components::skeleton::DashboardRowSkeleton;
use crate::models::account::Account;
use crate::models::subscription::Subscription;
use crate::state::rates::RatesState;
use crate::models::transaction::Transaction;
use crate::state::accounts::ACCOUNTS;
use crate::state::auth::AUTH;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::state::transactions::TRANSACTIONS;
use crate::storage::Storage;
use crate::utils::format::format_currency;
use crate::utils::icons::icon_display;

// ── Pure helpers (testable without Dioxus) ───────────────────────────────────

/// Convert a subscription amount to its monthly equivalent.
/// Weekly uses 52/12 weeks, matching the reports backend.
pub fn monthly_equivalent(amount: Decimal, frequency: &str) -> Decimal {
    match frequency {
        "daily" => amount * Decimal::from(30),
        "weekly" => amount * Decimal::from_str_exact("4.33").unwrap_or(Decimal::ZERO),
        "monthly" => amount,
        "quarterly" => amount / Decimal::from(3),
        "yearly" => amount / Decimal::from(12),
        _ => amount,
    }
}

/// Days in a month, clamped for leap years.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    let first_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 1).expect("valid month"));
    first_next
        .pred_opt()
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 1).expect("valid month"))
        .day()
}

/// Add months to a date, clamping the day (e.g. Jan 31 + 1 month = Feb 28/29).
pub fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let total = date.year() * 12 + date.month() as i32 - 1 + months;
    let year = total.div_euclid(12);
    let month = (total.rem_euclid(12) + 1) as u32;
    let day = date.day().min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
}

fn advance_date(date: NaiveDate, frequency: &str) -> NaiveDate {
    match frequency {
        "weekly" => date + chrono::Duration::weeks(1),
        "monthly" => add_months(date, 1),
        "quarterly" => add_months(date, 3),
        "yearly" => add_months(date, 12),
        _ => date + chrono::Duration::days(30),
    }
}

/// Actual subscription charge dates that fall inside the given month.
pub fn billing_dates_in_month(
    next_billing_date: Option<NaiveDate>,
    start_date: NaiveDate,
    frequency: &str,
    year: i32,
    month: u32,
) -> Vec<NaiveDate> {
    let month_start = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month start");
    let month_end =
        NaiveDate::from_ymd_opt(year, month, days_in_month(year, month)).expect("valid month end");

    let mut date = next_billing_date.unwrap_or(start_date);
    let mut guard = 0;
    while date < month_start && guard < 1000 {
        date = advance_date(date, frequency);
        guard += 1;
    }

    let mut dates = Vec::new();
    while date <= month_end && guard < 2000 {
        dates.push(date);
        date = advance_date(date, frequency);
        guard += 1;
    }
    dates
}

fn add_months_with_day(date: NaiveDate, months: i32, day: i16) -> NaiveDate {
    let total = date.year() * 12 + date.month() as i32 - 1 + months;
    let year = total.div_euclid(12);
    let month = (total.rem_euclid(12) + 1) as u32;
    let day = (day.max(1) as u32).min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
}

fn advance_scheduled(
    date: NaiveDate,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
) -> NaiveDate {
    match frequency {
        "daily" => date + chrono::Duration::days(1),
        "weekly" => date + chrono::Duration::weeks(1),
        "monthly" => add_months_with_day(date, 1, schedule_day.unwrap_or(date.day() as i16)),
        "quarterly" => add_months_with_day(date, 3, schedule_day.unwrap_or(date.day() as i16)),
        "yearly" => {
            let year = date.year() + 1;
            let month = schedule_month.unwrap_or(date.month() as i16) as u32;
            let day = (schedule_day.unwrap_or(date.day() as i16).max(1) as u32)
                .min(days_in_month(year, month));
            NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
        }
        _ => date + chrono::Duration::days(30),
    }
}

fn first_scheduled_on_or_after(
    start_date: NaiveDate,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
) -> NaiveDate {
    match frequency {
        "weekly" => {
            let target = schedule_day.unwrap_or(start_date.weekday().number_from_monday() as i16);
            let current = start_date.weekday().number_from_monday() as i16;
            start_date + chrono::Duration::days((target - current).rem_euclid(7) as i64)
        }
        "monthly" | "quarterly" => {
            let day = schedule_day.unwrap_or(start_date.day() as i16);
            let candidate = NaiveDate::from_ymd_opt(
                start_date.year(),
                start_date.month(),
                (day.max(1) as u32).min(days_in_month(start_date.year(), start_date.month())),
            )
            .unwrap_or(start_date);
            if candidate < start_date {
                add_months_with_day(candidate, 1, day)
            } else {
                candidate
            }
        }
        "yearly" => {
            let day = schedule_day.unwrap_or(start_date.day() as i16);
            let month = schedule_month.unwrap_or(start_date.month() as i16) as u32;
            let candidate = NaiveDate::from_ymd_opt(
                start_date.year(),
                month,
                (day.max(1) as u32).min(days_in_month(start_date.year(), month)),
            )
            .unwrap_or(start_date);
            if candidate < start_date {
                let next_year = start_date.year() + 1;
                NaiveDate::from_ymd_opt(
                    next_year,
                    month,
                    (day.max(1) as u32).min(days_in_month(next_year, month)),
                )
                .unwrap_or(start_date)
            } else {
                candidate
            }
        }
        _ => start_date,
    }
}

pub fn first_billing_starts_in_month(
    start_date: NaiveDate,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
    year: i32,
    month: u32,
) -> bool {
    let first = first_scheduled_on_or_after(start_date, frequency, schedule_day, schedule_month);
    first.year() == year && first.month() == month
}

/// Charge dates including the configured recurrence schedule and end date.
pub fn billing_dates_in_month_scheduled(
    next_billing_date: Option<NaiveDate>,
    frequency: &str,
    schedule_day: Option<i16>,
    schedule_month: Option<i16>,
    end_date: Option<NaiveDate>,
    year: i32,
    month: u32,
) -> Vec<NaiveDate> {
    let month_start = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month start");
    let month_end =
        NaiveDate::from_ymd_opt(year, month, days_in_month(year, month)).expect("valid month end");

    let Some(mut date) = next_billing_date else {
        return Vec::new();
    };
    let mut guard = 0;
    while date < month_start && guard < 1000 {
        date = advance_scheduled(date, frequency, schedule_day, schedule_month);
        guard += 1;
    }

    let mut dates = Vec::new();
    while date <= month_end && guard < 2000 {
        if end_date.is_some_and(|end| date >= end) {
            break;
        }
        dates.push(date);
        date = advance_scheduled(date, frequency, schedule_day, schedule_month);
        guard += 1;
    }
    dates
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountBudget {
    pub account_id: Uuid,
    pub name: String,
    pub color: String,
    pub icon: String,
    pub subscriptions: Decimal,
    pub manual: Decimal,
    pub budget: Decimal,
    pub pending: Decimal,
    pub subscription_count: usize,
    pub manual_count: usize,
    pub pending_count: usize,
}

impl AccountBudget {
    pub fn has_movements(&self) -> bool {
        self.budget > Decimal::ZERO
            || self.pending > Decimal::ZERO
            || self.subscription_count > 0
            || self.manual_count > 0
    }
}

/// Build the monthly transfer plan per account. `today` is injected so the
/// pending calculation is deterministic in tests.
pub fn build_account_budgets(
    accounts: &[Account],
    subscriptions: &[Subscription],
    transactions: &[Transaction],
    today: NaiveDate,
    rates: &RatesState,
) -> Vec<AccountBudget> {
    let mut plan: HashMap<Uuid, AccountBudget> = accounts
        .iter()
        .map(|account| {
            (
                account.id,
                AccountBudget {
                    account_id: account.id,
                    name: account.name.clone(),
                    color: account
                        .color
                        .clone()
                        .unwrap_or_else(|| "#6b7280".to_string()),
                    icon: account.icon.clone().unwrap_or_else(|| "Tag".to_string()),
                    subscriptions: Decimal::ZERO,
                    manual: Decimal::ZERO,
                    budget: Decimal::ZERO,
                    pending: Decimal::ZERO,
                    subscription_count: 0,
                    manual_count: 0,
                    pending_count: 0,
                },
            )
        })
        .collect();

    for sub in subscriptions.iter().filter(|s| s.is_active) {
        let occurrences = crate::utils::recurrence::occurrences_in_month(
            sub.start_date,
            &sub.recurrence,
            sub.end_date,
            today.year(),
            today.month(),
            sub.amount,
        );
        if occurrences.is_empty() {
            continue;
        }
        let entry = plan.entry(sub.account_id).or_insert_with(|| AccountBudget {
            account_id: sub.account_id,
            name: "No account".to_string(),
            color: "#6b7280".to_string(),
            icon: "Tag".to_string(),
            subscriptions: Decimal::ZERO,
            manual: Decimal::ZERO,
            budget: Decimal::ZERO,
            pending: Decimal::ZERO,
            subscription_count: 0,
            manual_count: 0,
            pending_count: 0,
        });

        entry.subscription_count += 1;
        for (date, amount) in occurrences {
            let amount = rates.convert(amount, &sub.currency);
            entry.subscriptions += amount;
            entry.budget += amount;
            if date >= today {
                entry.pending += amount;
                entry.pending_count += 1;
            }
        }
    }

    for txn in transactions
        .iter()
        .filter(|t| t.date.year() == today.year() && t.date.month() == today.month())
    {
        let entry = plan.entry(txn.account_id).or_insert_with(|| AccountBudget {
            account_id: txn.account_id,
            name: "No account".to_string(),
            color: "#6b7280".to_string(),
            icon: "Tag".to_string(),
            subscriptions: Decimal::ZERO,
            manual: Decimal::ZERO,
            budget: Decimal::ZERO,
            pending: Decimal::ZERO,
            subscription_count: 0,
            manual_count: 0,
            pending_count: 0,
        });
        let amount = rates.convert(txn.amount, &txn.currency);
        entry.manual += amount;
        entry.budget += amount;
        entry.manual_count += 1;
        // A manual payment recorded today is already paid; only future-dated
        // manual payments remain pending.
        if txn.date > today {
            entry.pending += amount;
            entry.pending_count += 1;
        }
    }

    let mut result: Vec<AccountBudget> = plan.into_values().collect();
    result.sort_by(|a, b| {
        b.budget
            .cmp(&a.budget)
            .then_with(|| b.pending.cmp(&a.pending))
            .then_with(|| a.name.cmp(&b.name))
    });
    result
}

pub fn total_budget(plan: &[AccountBudget]) -> Decimal {
    plan.iter().map(|b| b.budget).sum()
}

pub fn total_pending(plan: &[AccountBudget]) -> Decimal {
    plan.iter().map(|b| b.pending).sum()
}

/// Share of the fixed budget that is still pending from today, capped at 100.
pub fn pending_ratio(budget: Decimal, pending: Decimal) -> f64 {
    if budget <= Decimal::ZERO {
        return 0.0;
    }
    let ratio = pending / budget * Decimal::from(100);
    ratio
        .min(Decimal::from(100))
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0)
}

/// Format a billing date into a human-readable relative string.
pub fn next_billing_days_text(date: Option<NaiveDate>) -> Option<String> {
    date.map(|d| {
        let now = Local::now().date_naive();
        let diff = (d - now).num_days();
        if diff == 0 {
            "Today".into()
        } else if diff == 1 {
            "Tomorrow".into()
        } else if diff < 0 {
            "Due".into()
        } else {
            format!("{} days", diff)
        }
    })
}

pub fn format_date_short_fn(d: NaiveDate) -> String {
    d.format("%a, %b %-d").to_string()
}

pub fn is_today_fn(d: NaiveDate) -> bool {
    d == Local::now().date_naive()
}

// ── Dashboard Component ──────────────────────────────────────────────────────

#[component]
pub fn Dashboard() -> Element {
    use_hook(move || {
        spawn(async move {
            if ACCOUNTS().accounts.is_empty() {
                ACCOUNTS.write().set_loading();
            }
            if SUBSCRIPTIONS().subscriptions.is_empty() {
                SUBSCRIPTIONS.write().set_loading();
            }
            if TRANSACTIONS().transactions.is_empty() {
                TRANSACTIONS.write().set_loading();
            }

            match Storage::list_accounts().await {
                Ok(data) => ACCOUNTS.write().set_data(data),
                Err(e) => ACCOUNTS.write().set_error(e.to_string()),
            }
            match Storage::list_subscriptions().await {
                Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
            }
            match Storage::list_transactions().await {
                Ok(data) => TRANSACTIONS.write().set_data(data),
                Err(e) => TRANSACTIONS.write().set_error(e.to_string()),
            }
        });
    });

    let accounts_state = ACCOUNTS();
    let sub_state = SUBSCRIPTIONS();
    let txns_state = TRANSACTIONS();
    let auth_state = AUTH();

    let today = Local::now().date_naive();
    let plan = build_account_budgets(
        &accounts_state.accounts,
        &sub_state.subscriptions,
        &txns_state.transactions,
        today,
        &crate::state::rates::RATES(),
    );
    let visible: Vec<AccountBudget> = plan.clone();

    let budget_total = total_budget(&visible);
    let pending_total = total_pending(&visible);
    let pending_count: usize = visible.iter().map(|b| b.pending_count).sum();
    let accounts_with_pending = visible.iter().filter(|b| b.pending > Decimal::ZERO).count();
    let manual_count: usize = visible.iter().map(|b| b.manual_count).sum();
    let active_count = sub_state.active().len();
    let ratio = pending_ratio(budget_total, pending_total);
    let over_budget = pending_total > budget_total;
    let days_left = days_in_month(today.year(), today.month()).saturating_sub(today.day());
    let month_label = today.format("%B %Y").to_string();

    let currency = auth_state.currency().to_string();
    let user_name = auth_state
        .user
        .as_ref()
        .and_then(|u| u.name.clone())
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| "there".to_string());

    let mut upcoming: Vec<_> = sub_state
        .active()
        .into_iter()
        .filter(|s| s.next_billing_date.is_some())
        .collect();
    upcoming.sort_by_key(|s| s.next_billing_date);
    let upcoming_top5: Vec<_> = upcoming.iter().take(5).collect();

    let mut hovered_account = use_signal(|| None::<Uuid>);

    let mut manual_payments: Vec<_> = txns_state
        .transactions
        .iter()
        .filter(|t| t.date.year() == today.year() && t.date.month() == today.month())
        .collect();
    manual_payments.sort_by_key(|t| std::cmp::Reverse(t.date));
    let manual_top4: Vec<_> = manual_payments.iter().take(4).collect();

    let is_loading = (sub_state.is_loading && sub_state.subscriptions.is_empty())
        || (accounts_state.is_loading && accounts_state.accounts.is_empty())
        || (txns_state.is_loading && txns_state.transactions.is_empty());

    let nav = navigator();

    rsx! {
        div { class: "p-4 md:p-8 pb-24 lg:pb-8 flex flex-col gap-6 animate-in fade-in duration-300",
            // Header
            div { class: "flex flex-col md:flex-row md:items-end md:justify-between gap-4",
                div {
                    p { class: "text-sm text-muted-foreground font-medium", "Welcome back, {user_name}" }
                    h1 { class: "text-3xl md:text-4xl font-bold tracking-tighter text-foreground mt-1",
                        "This month's plan"
                    }
                    p { class: "text-sm text-muted-foreground mt-2",
                        "What to transfer into each account, and what is still left to pay."
                    }
                }
                div { class: "hidden md:flex flex-wrap gap-2",
                    {
                        let nav1 = nav;
                        rsx! {
                            a {
                                href: "/subscriptions",
                                class: "inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2.5 text-sm font-semibold text-primary-foreground hover:bg-primary/90 active:scale-[0.97] transition-all duration-200",
                                onclick: move |e| {
                                    e.prevent_default();
                                    *crate::state::create_intent::CREATE_INTENT.write() =
                                        Some(crate::state::create_intent::CreateIntent::Subscription);
                                    nav1.push(crate::Route::Subscriptions {});
                                },
                                svg {
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    class: "h-4 w-4",
                                    line { x1: "12", y1: "5", x2: "12", y2: "19" }
                                    line { x1: "5", y1: "12", x2: "19", y2: "12" }
                                }
                                "Add subscription"
                            }
                        }
                    }
                    {
                        let nav2 = nav;
                        rsx! {
                            a {
                                href: "/transactions",
                                class: "inline-flex items-center gap-2 rounded-lg border border-border bg-card px-4 py-2.5 text-sm font-semibold text-foreground hover:bg-accent active:scale-[0.97] transition-all duration-200",
                                onclick: move |e| {
                                    e.prevent_default();
                                    *crate::state::create_intent::CREATE_INTENT.write() =
                                        Some(crate::state::create_intent::CreateIntent::Transaction);
                                    nav2.push(crate::Route::Transactions {});
                                },
                                svg {
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    class: "h-4 w-4",
                                    line { x1: "12", y1: "1", x2: "12", y2: "23" }
                                    path { d: "M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 1 1 0 7H6" }
                                }
                                "Add transaction"
                            }
                        }
                    }
                    {
                        let nav3 = nav;
                        rsx! {
                            a {
                                href: "/debts",
                                class: "inline-flex items-center gap-2 rounded-lg border border-border bg-card px-4 py-2.5 text-sm font-semibold text-foreground hover:bg-accent active:scale-[0.97] transition-all duration-200",
                                onclick: move |e| {
                                    e.prevent_default();
                                    *crate::state::create_intent::CREATE_INTENT.write() =
                                        Some(crate::state::create_intent::CreateIntent::Debt);
                                    nav3.push(crate::Route::Debts {});
                                },
                                svg {
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    class: "h-4 w-4",
                                    line { x1: "12", y1: "5", x2: "12", y2: "19" }
                                    line { x1: "5", y1: "12", x2: "19", y2: "12" }
                                }
                                "Add debt"
                            }
                        }
                    }
                }
            }

            // Hero: fixed budget + pending from today
            div { class: "rounded-2xl border border-border/60 bg-card overflow-hidden",
                div { class: "grid lg:grid-cols-[1.15fr_0.85fr]",
                    div { class: "p-6 md:p-8",
                        p { class: "text-xs text-muted-foreground uppercase tracking-[0.18em] font-semibold",
                            "Plan for {month_label}"
                        }
                        p { class: "text-sm text-muted-foreground mt-5", "Transfer this month" }
                        div { class: "text-4xl md:text-6xl font-bold text-foreground tracking-tight font-mono tabular-nums mt-1",
                            "{format_currency(budget_total, &currency)}"
                        }
                        p { class: "text-xs text-muted-foreground mt-2",
                            "Actual charges this month + manual payments."
                        }
                        div { class: "flex flex-wrap items-center gap-x-5 gap-y-2 mt-6 text-xs text-muted-foreground",
                            span { class: "inline-flex items-center gap-1.5",
                                span { class: "h-1.5 w-1.5 rounded-full bg-primary" }
                                "{visible.len()} accounts"
                            }
                            span { "{active_count} active subscriptions" }
                            span { "{manual_count} manual payments" }
                        }
                    }
                    div { class: "border-t lg:border-t-0 lg:border-l border-border/60 p-6 md:p-8 bg-muted/20",
                        p { class: "text-xs text-muted-foreground uppercase tracking-[0.18em] font-semibold",
                            "Pending from today"
                        }
                        div {
                            class: if pending_total > Decimal::ZERO { "text-3xl md:text-4xl font-bold tracking-tight font-mono tabular-nums mt-5 text-foreground" } else { "text-3xl md:text-4xl font-bold tracking-tight font-mono tabular-nums mt-5 text-muted-foreground" },
                            "{format_currency(pending_total, &currency)}"
                        }
                        p { class: "text-xs text-muted-foreground mt-2",
                            {if pending_count == 0 {
                                "All covered through today.".to_string()
                            } else if days_left == 0 {
                                format!("{} payments · last day of the month", pending_count)
                            } else {
                                format!("{} payments · {} days left", pending_count, days_left)
                            }}
                        }
                        div { class: "mt-5 h-2 rounded-full bg-muted overflow-hidden",
                            div {
                                class: if over_budget { "h-full rounded-full bg-amber-500" } else { "h-full rounded-full bg-primary" },
                                style: "width: {ratio:.0}%",
                            }
                        }
                        p { class: "text-[11px] text-muted-foreground mt-2",
                            {if budget_total <= Decimal::ZERO {
                                "No budget this month.".to_string()
                            } else if over_budget {
                                format!(
                                    "Over the plan by {}",
                                    format_currency(pending_total - budget_total, &currency)
                                )
                            } else {
                                format!("{:.0}% of the budget pending", ratio)
                            }}
                        }
                    }
                }
            }

            // Account ledger
            div { class: "rounded-xl border border-border/60 bg-card overflow-hidden",
                div { class: "p-5 md:p-6 pb-3 flex flex-col sm:flex-row sm:items-end sm:justify-between gap-2",
                    div {
                        h2 { class: "text-sm font-semibold text-muted-foreground uppercase tracking-[0.16em]",
                            "Transfer by account"
                        }
                        p { class: "text-sm text-muted-foreground mt-1",
                            "Actual charges this month and pending from today."
                        }
                    }
                    p { class: "text-xs text-muted-foreground",
                        "{accounts_with_pending} accounts with pending payments"
                    }
                }

                if is_loading {
                    div { class: "space-y-3 p-5 md:p-6 pt-2",
                        for _ in 0..4 {
                            DashboardRowSkeleton {}
                        }
                    }
                } else if visible.is_empty() {
                    div { class: "p-8 flex flex-col items-center justify-center gap-3 text-center",
                        div { class: "h-12 w-12 rounded-full bg-muted flex items-center justify-center",
                            {icon_display("Wallet")}
                        }
                        p { class: "text-base font-medium", "No movements this month" }
                        p { class: "text-sm text-muted-foreground max-w-md",
                            "Add a subscription or manual payment and the per-account plan will appear here."
                        }
                        {
                            let nav3 = nav;
                            rsx! {
                                button {
                                    class: "hidden md:inline-flex mt-2 items-center gap-2 h-10 px-4 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90",
                                    onclick: move |_| {
                                        *crate::state::create_intent::CREATE_INTENT.write() =
                                            Some(crate::state::create_intent::CreateIntent::Subscription);
                                        nav3.push(crate::Route::Subscriptions {});
                                    },
                                    "Add subscription"
                                }
                            }
                        }
                    }
                } else {
                    div { class: "hidden sm:grid grid-cols-[minmax(0,1fr)_9rem_9rem] gap-4 px-5 md:px-6 py-2 border-y border-border/50 bg-muted/20",
                        span { class: "text-[10px] uppercase tracking-[0.16em] text-muted-foreground font-semibold", "Account" }
                        span { class: "text-[10px] uppercase tracking-[0.16em] text-muted-foreground font-semibold text-right", "Budget" }
                        span { class: "text-[10px] uppercase tracking-[0.16em] text-muted-foreground font-semibold text-right", "Pending" }
                    }
                    div { class: "divide-y divide-border/50",
                        for account in visible.iter() {
                            {
                                let color = account.color.clone();
                                let icon = account.icon.clone();
                                let name = account.name.clone();
                                let account_id = account.account_id;
                                let pending_class = if account.pending > Decimal::ZERO {
                                    "text-sm font-semibold font-mono tabular-nums text-foreground"
                                } else {
                                    "text-sm font-semibold font-mono tabular-nums text-muted-foreground"
                                };
                                let breakdown = if account.manual > Decimal::ZERO {
                                    format!(
                                        "{} subscriptions · {} manual",
                                        account.subscription_count, account.manual_count
                                    )
                                } else {
                                    format!("{} subscriptions", account.subscription_count)
                                };
                                rsx! {
                                    div {
                                        key: "{account.account_id}",
                                        class: "px-5 md:px-6 py-4 grid grid-cols-2 sm:grid-cols-[minmax(0,1fr)_9rem_9rem] gap-x-4 gap-y-3 items-center hover:bg-muted/20 focus:bg-muted/20 focus:outline-none transition-colors",
                                        tabindex: "0",
                                        onmouseenter: move |_| hovered_account.set(Some(account_id)),
                                        onmouseleave: move |_| hovered_account.set(None),
                                        onfocus: move |_| hovered_account.set(Some(account_id)),
                                        onblur: move |_| hovered_account.set(None),
                                        div { class: "col-span-2 sm:col-span-1 flex items-center gap-3 min-w-0",
                                            div {
                                                class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-lg",
                                                style: "background-color: {color}20; color: {color}",
                                                {icon_display(&icon)}
                                            }
                                            div { class: "min-w-0",
                                                p { class: "text-sm font-semibold text-foreground truncate", {name} }
                                                p { class: "text-xs text-muted-foreground truncate", {breakdown} }
                                            }
                                        }
                                        div { class: "text-left sm:text-right",
                                            p { class: "sm:hidden text-[10px] uppercase tracking-wider text-muted-foreground", "Budget" }
                                            p { class: "text-sm font-semibold font-mono tabular-nums text-foreground",
                                                "{format_currency(account.budget, &currency)}"
                                            }
                                            p { class: "text-[11px] text-muted-foreground",
                                                "{format_currency(account.subscriptions, &currency)} recurring"
                                            }
                                        }
                                        div { class: "text-right",
                                            p { class: "sm:hidden text-[10px] uppercase tracking-wider text-muted-foreground", "Pending" }
                                            p { class: "{pending_class}",
                                                "{format_currency(account.pending, &currency)}"
                                            }
                                            p { class: "text-[11px] text-muted-foreground",
                                                {if account.pending_count == 0 {
                                                    "Up to date".to_string()
                                                } else {
                                                    format!("{} payments", account.pending_count)
                                                }}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "px-5 md:px-6 py-4 bg-muted/30 border-t border-border/50 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                        div {
                            p { class: "text-xs uppercase tracking-[0.16em] text-muted-foreground font-semibold", "Month total" }
                            p { class: "text-xs text-muted-foreground",
                                {format!("{} accounts", visible.len())}
                            }
                        }
                        div { class: "flex items-center gap-8 sm:gap-12",
                            div { class: "text-left sm:text-right",
                                p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "Budget" }
                                p { class: "text-base font-bold font-mono tabular-nums", "{format_currency(budget_total, &currency)}" }
                            }
                            div { class: "text-right",
                                p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "Pending" }
                                p { class: "text-base font-bold font-mono tabular-nums", "{format_currency(pending_total, &currency)}" }
                            }
                        }
                    }
                }
            }

            // Secondary: upcoming payments + manual payments
            div { class: "grid gap-4 lg:grid-cols-3 items-start",
                div { class: "lg:col-span-2 rounded-xl bg-card border border-border/60",
                    div { class: "p-5 md:p-6 pb-2 flex items-center justify-between",
                        div {
                            h3 { class: "text-sm font-semibold text-muted-foreground uppercase tracking-[0.16em]", "Upcoming payments" }
                            p { class: "text-xs text-muted-foreground mt-1", "Subscriptions with a billing date." }
                        }
                        {
                            let nav4 = nav;
                            rsx! {
                                button {
                                    class: "text-xs font-medium text-primary hover:underline",
                                    onclick: move |_| { nav4.push(crate::Route::Calendar {}); },
                                    "View calendar"
                                }
                            }
                        }
                    }
                    if upcoming_top5.is_empty() {
                        p { class: "p-6 pt-2 text-sm text-muted-foreground", "No upcoming payments." }
                    } else {
                        div { class: "px-5 md:px-6 pb-4 space-y-1",
                            for sub in upcoming_top5 {
                                {
                                    let next_date = sub.next_billing_date.unwrap();
                                    let today_flag = is_today_fn(next_date);
                                    let date_str = format_date_short_fn(next_date);
                                    let days_label = next_billing_days_text(Some(next_date));
                                    let logo_url = sub.logo_url.clone();
                                    let account_name = ACCOUNTS().name_for(&sub.account_id);
                                    rsx! {
                                        {
                                            let linked = hovered_account() == Some(sub.account_id);
                                            let row_class = if linked { "flex items-center justify-between py-2.5 -mx-2 px-2 rounded-lg bg-primary/5 transition-colors" } else { "flex items-center justify-between py-2.5 -mx-2 px-2 rounded-lg transition-colors" };
                                            rsx! {
                                        div { key: "{sub.id}", class: "{row_class}",
                                            div { class: "flex items-center gap-3 min-w-0",
                                                div { class: "flex h-9 w-9 shrink-0 items-center justify-center rounded-lg overflow-hidden bg-secondary",
                                                    {logo_image_with_fallback(&format!("upcoming-{}", sub.id), logo_url.as_deref(), &sub.name, "h-9 w-9 object-contain")}
                                                }
                                                div { class: "min-w-0",
                                                    p { class: "text-sm font-medium text-foreground truncate", {sub.name.to_string()} }
                                                    p { class: "text-xs text-muted-foreground truncate",
                                                        "{account_name} · {date_str}"
                                                    }
                                                }
                                            }
                                            div { class: "text-right shrink-0 ml-4",
                                                p { class: "text-sm font-semibold font-mono tabular-nums text-foreground",
                                                    ConvertedAmount { amount: sub.amount, currency: sub.currency.clone() }
                                                }
                                                if let Some(label) = days_label {
                                                    p {
                                                        class: if today_flag { "text-xs font-medium text-primary" } else { "text-xs text-muted-foreground" },
                                                        {label.to_string()}
                                                    }
                                                }
                                            }
                                        }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "rounded-xl bg-card border border-border/60",
                    div { class: "p-5 md:p-6 pb-2 flex items-center justify-between",
                        div {
                            h3 { class: "text-sm font-semibold text-muted-foreground uppercase tracking-[0.16em]", "Manual payments" }
                            p { class: "text-xs text-muted-foreground mt-1", "This month." }
                        }
                        {
                            let nav5 = nav;
                            rsx! {
                                button {
                                    class: "text-xs font-medium text-primary hover:underline",
                                    onclick: move |_| { nav5.push(crate::Route::Transactions {}); },
                                    "View all"
                                }
                            }
                        }
                    }
                    if manual_top4.is_empty() {
                        p { class: "p-6 pt-2 text-sm text-muted-foreground", "No manual payments this month." }
                    } else {
                        div { class: "px-5 md:px-6 pb-4 space-y-2",
                            for txn in manual_top4 {
                                {
                                    let account_name = ACCOUNTS().name_for(&txn.account_id);
                                    rsx! {
                                        {
                                            let linked = hovered_account() == Some(txn.account_id);
                                            let row_class = if linked { "flex items-center justify-between gap-3 py-1.5 -mx-2 px-2 rounded-lg bg-primary/5 transition-colors" } else { "flex items-center justify-between gap-3 py-1.5 -mx-2 px-2 rounded-lg transition-colors" };
                                            rsx! {
                                        div { key: "{txn.id}", class: "{row_class}",
                                            div { class: "min-w-0",
                                                p { class: "text-sm text-foreground truncate", {txn.name.to_string()} }
                                                p { class: "text-xs text-muted-foreground truncate",
                                                    "{account_name} · {txn.date.format(\"%b %-d\")}"
                                                }
                                            }
                                            p { class: "text-sm font-semibold font-mono tabular-nums shrink-0",
                                                ConvertedAmount { amount: txn.amount, currency: txn.currency.clone() }
                                            }
                                        }
                                            }
                                        }
                                    }
                                }
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
    use crate::models::subscription::Recurrence;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
    }

    fn account(name: &str) -> Account {
        Account {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: name.into(),
            icon: None,
            color: None,
            is_default: false,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    fn subscription(
        account_id: Uuid,
        amount: Decimal,
        frequency: &str,
        start_date: NaiveDate,
        next_billing_date: Option<NaiveDate>,
        is_active: bool,
    ) -> Subscription {
        Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            account_id,
            category_id: Uuid::new_v4(),
            name: "Test".into(),
            amount,
            currency: "EUR".into(),
            frequency: frequency.into(),
            schedule_day: None,
            schedule_month: None,
            recurrence: Recurrence::from_legacy(frequency, start_date, None, None),
            start_date,
            end_date: None,
            next_billing_date,
            is_active,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    fn transaction(account_id: Uuid, amount: Decimal, date: NaiveDate) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            account_id,
            category_id: Uuid::new_v4(),
            name: "Manual".into(),
            amount,
            currency: "EUR".into(),
            date,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn monthly_equivalent_normalizes_frequencies() {
        assert_eq!(monthly_equivalent(dec!(15), "monthly"), dec!(15));
        assert_eq!(monthly_equivalent(dec!(30), "quarterly"), dec!(10));
        assert_eq!(monthly_equivalent(dec!(120), "yearly"), dec!(10));
        assert_eq!(monthly_equivalent(dec!(5), "weekly"), dec!(21.65));
        assert_eq!(monthly_equivalent(dec!(5), "biweekly"), dec!(5));
    }

    #[test]
    fn add_months_clamps_end_of_month() {
        assert_eq!(add_months(date(2024, 1, 31), 1), date(2024, 2, 29));
        assert_eq!(add_months(date(2023, 1, 31), 1), date(2023, 2, 28));
        assert_eq!(add_months(date(2024, 12, 15), 1), date(2025, 1, 15));
    }

    #[test]
    fn billing_dates_monthly_inside_month() {
        let dates = billing_dates_in_month(
            Some(date(2024, 6, 15)),
            date(2024, 1, 15),
            "monthly",
            2024,
            6,
        );
        assert_eq!(dates, vec![date(2024, 6, 15)]);
    }

    #[test]
    fn billing_dates_weekly_multiple() {
        let dates =
            billing_dates_in_month(Some(date(2024, 6, 3)), date(2024, 6, 3), "weekly", 2024, 6);
        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], date(2024, 6, 3));
        assert_eq!(dates[3], date(2024, 6, 24));
    }

    #[test]
    fn billing_dates_yearly_only_in_billing_month() {
        let none =
            billing_dates_in_month(Some(date(2024, 7, 1)), date(2024, 1, 1), "yearly", 2024, 6);
        assert!(none.is_empty());
        let some =
            billing_dates_in_month(Some(date(2024, 6, 10)), date(2024, 1, 1), "yearly", 2024, 6);
        assert_eq!(some, vec![date(2024, 6, 10)]);
    }

    #[test]
    fn scheduled_billing_respects_end_date() {
        let end = Some(date(2024, 3, 1));
        let feb = billing_dates_in_month_scheduled(
            Some(date(2024, 1, 15)),
            "monthly",
            Some(15),
            None,
            end,
            2024,
            2,
        );
        assert_eq!(feb, vec![date(2024, 2, 15)]);
        let march = billing_dates_in_month_scheduled(
            Some(date(2024, 1, 15)),
            "monthly",
            Some(15),
            None,
            end,
            2024,
            3,
        );
        assert!(march.is_empty());
    }

    #[test]
    fn scheduled_yearly_keeps_configured_month() {
        let dates = billing_dates_in_month_scheduled(
            Some(date(2024, 6, 10)),
            "yearly",
            Some(10),
            Some(6),
            None,
            2025,
            6,
        );
        assert_eq!(dates, vec![date(2025, 6, 10)]);
    }

    #[test]
    fn past_billing_dates_are_never_overdue() {
        let past = Local::now().date_naive() - chrono::Duration::days(2);
        assert_eq!(next_billing_days_text(Some(past)).as_deref(), Some("Due"));
    }

    #[test]
    fn dashboard_add_actions_are_desktop_only() {
        let src = include_str!("dashboard.rs");
        assert!(src.contains("hidden md:flex flex-wrap gap-2"));
        assert!(src.contains("hidden md:inline-flex mt-2"));
    }

    #[test]
    fn first_billing_month_follows_start_activation() {
        let start_past = date(2024, 11, 3);
        let start_future = date(2024, 11, 12);
        assert!(first_billing_starts_in_month(
            start_past,
            "monthly",
            Some(3),
            None,
            2024,
            11
        ));
        assert!(!first_billing_starts_in_month(
            start_future,
            "monthly",
            Some(3),
            None,
            2024,
            11
        ));
        assert!(first_billing_starts_in_month(
            start_future,
            "monthly",
            Some(3),
            None,
            2024,
            12
        ));
    }

    #[test]
    fn build_plan_splits_subscriptions_and_manual_payments() {
        let main = account("Main");
        let savings = account("Savings");
        let main_id = main.id;
        let savings_id = savings.id;
        let today = date(2024, 6, 15);

        let subs = vec![
            subscription(
                main_id,
                dec!(10),
                "monthly",
                date(2024, 6, 20),
                Some(date(2024, 6, 20)),
                true,
            ),
            subscription(
                savings_id,
                dec!(120),
                "yearly",
                date(2024, 6, 10),
                Some(date(2024, 6, 10)),
                true,
            ),
            subscription(
                main_id,
                dec!(99),
                "monthly",
                date(2024, 6, 20),
                Some(date(2024, 6, 20)),
                false,
            ),
        ];
        let txns = vec![
            transaction(main_id, dec!(5), date(2024, 6, 10)),
            transaction(main_id, dec!(7), date(2024, 6, 20)),
            transaction(main_id, dec!(8), date(2024, 5, 20)),
        ];

        let plan = build_account_budgets(&[main, savings], &subs, &txns, today, &RatesState::default());
        let main_plan = plan.iter().find(|b| b.account_id == main_id).unwrap();
        let savings_plan = plan.iter().find(|b| b.account_id == savings_id).unwrap();

        assert_eq!(main_plan.subscriptions, dec!(10));
        assert_eq!(main_plan.manual, dec!(12));
        assert_eq!(main_plan.budget, dec!(22));
        assert_eq!(main_plan.pending, dec!(17));
        assert_eq!(main_plan.pending_count, 2);

        let today_txn = transaction(main_id, dec!(4), today);
        let plan_with_today = build_account_budgets(&[account("Main")], &[], &[today_txn], today, &RatesState::default());
        assert_eq!(plan_with_today[0].budget, dec!(4));
        assert_eq!(plan_with_today[0].pending, dec!(0));

        assert_eq!(savings_plan.subscriptions, dec!(120));
        assert_eq!(savings_plan.pending, dec!(0));
        assert_eq!(savings_plan.budget, dec!(120));
    }

    #[test]
    fn build_plan_includes_unassigned_bucket() {
        let today = date(2024, 6, 15);
        let orphan = subscription(
            Uuid::nil(),
            dec!(10),
            "monthly",
            date(2024, 6, 20),
            Some(date(2024, 6, 20)),
            true,
        );
        let plan = build_account_budgets(&[], &[orphan], &[], today, &RatesState::default());
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].name, "No account");
        assert_eq!(plan[0].pending, dec!(10));
    }


    #[test]
    fn dashboard_converts_foreign_currency_totals() {
        let account = account("Main");
        let account_id = account.id;
        let today = date(2026, 5, 10);
        let mut usd = subscription(
            account_id,
            dec!(11),
            "monthly",
            date(2026, 5, 1),
            Some(date(2026, 5, 1)),
            true,
        );
        usd.currency = "USD".into();
        let mut rates = RatesState::default();
        rates.base = "EUR".into();
        rates.rates.insert("USD".into(), dec!(1.1));
        let plan = build_account_budgets(&[account], &[usd], &[], today, &rates);
        assert_eq!(plan[0].budget, dec!(10));
    }

    #[test]
    fn totals_and_ratio() {
        let plan = vec![
            AccountBudget {
                account_id: Uuid::new_v4(),
                name: "A".into(),
                color: "#000".into(),
                icon: "Tag".into(),
                subscriptions: dec!(10),
                manual: dec!(5),
                budget: dec!(15),
                pending: dec!(7.5),
                subscription_count: 1,
                manual_count: 1,
                pending_count: 1,
            },
            AccountBudget {
                account_id: Uuid::new_v4(),
                name: "B".into(),
                color: "#000".into(),
                icon: "Tag".into(),
                subscriptions: dec!(20),
                manual: dec!(0),
                budget: dec!(20),
                pending: dec!(30),
                subscription_count: 1,
                manual_count: 0,
                pending_count: 1,
            },
        ];
        assert_eq!(total_budget(&plan), dec!(35));
        assert_eq!(total_pending(&plan), dec!(37.5));
        assert_eq!(pending_ratio(dec!(35), dec!(37.5)), 100.0);
        assert_eq!(pending_ratio(dec!(20), dec!(10)), 50.0);
        assert_eq!(pending_ratio(dec!(0), dec!(10)), 0.0);
    }

    #[test]
    fn has_movements_ignores_empty_accounts() {
        let empty = AccountBudget {
            account_id: Uuid::new_v4(),
            name: "Empty".into(),
            color: "#000".into(),
            icon: "Tag".into(),
            subscriptions: Decimal::ZERO,
            manual: Decimal::ZERO,
            budget: Decimal::ZERO,
            pending: Decimal::ZERO,
            subscription_count: 0,
            manual_count: 0,
            pending_count: 0,
        };
        assert!(!empty.has_movements());
    }
}
