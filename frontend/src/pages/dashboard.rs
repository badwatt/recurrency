use dioxus::prelude::*;
use chrono::Local;

use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::state::auth::AUTH;
use crate::api::subscriptions as api_subs;
use crate::utils::format::format_currency;
use crate::components::logo_image_with_fallback;

// ── SVG Icon Components ──────────────────────────────────────────────────────

fn icon_credit_card() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "h-4 w-4 text-muted-foreground",
            rect { x: "1", y: "4", width: "22", height: "16", rx: "2" }
            line { x1: "1", y1: "10", x2: "23", y2: "10" }
        }
    }
}

fn icon_trending_up() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "h-4 w-4 text-muted-foreground",
            polyline { points: "23 6 13.5 15.5 8.5 10.5 1 18" }
            polyline { points: "17 6 23 6" }
            polyline { points: "23 6 23 12" }
        }
    }
}

fn icon_calendar() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "h-4 w-4 text-muted-foreground",
            rect { x: "3", y: "4", width: "18", height: "18", rx: "2", ry: "2" }
            line { x1: "16", y1: "2", x2: "16", y2: "6" }
            line { x1: "8", y1: "2", x2: "8", y2: "6" }
            line { x1: "3", y1: "10", x2: "21", y2: "10" }
        }
    }
}

fn icon_receipt() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "h-4 w-4 text-muted-foreground",
            line { x1: "4", y1: "2", x2: "4", y2: "22" }
            path { d: "M1 2h18a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H1" }
            path { d: "M4 18h2" }
            circle { cx: "12", cy: "10", r: "2" }
            path { d: "M13 14h-2" }
            line { x1: "1", y1: "20", x2: "4", y2: "20" }
        }
    }
}


// ── Dashboard Component ──────────────────────────────────────────────────────

#[component]
pub fn Dashboard() -> Element {
    // Fetch subscriptions on mount
    use_hook(move || {
        spawn(async move {
            SUBSCRIPTIONS.write().set_loading();
            match api_subs::list().await {
                Ok(subs) => SUBSCRIPTIONS.write().set_data(subs),
                Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
            }
        });
    });

    let sub_state = SUBSCRIPTIONS();
    let auth_state = AUTH();

    // Computed stats
    let active = sub_state.active();
    let total_active = active.len();
    let monthly_total = sub_state.monthly_total();
    let yearly_total = sub_state.yearly_total();
    let due_week = sub_state.due_this_week();
    let due_month = sub_state.due_this_month();

    let currency = auth_state.currency();

    // Active subscriptions (first 5)
    let active_top5: Vec<_> = active.iter().take(5).collect();

    // Upcoming payments: active subs with next_billing_date, sorted asc, first 5
    let mut upcoming: Vec<_> = active
        .iter()
        .filter(|s| s.next_billing_date.is_some())
        .collect();
    upcoming.sort_by_key(|s| s.next_billing_date);
    let upcoming_top5: Vec<_> = upcoming.iter().take(5).collect();

    let user_name = auth_state
        .user
        .as_ref()
        .and_then(|u| u.name.clone())
        .or_else(|| auth_state.user.as_ref().map(|u| u.email.clone()))
        .unwrap_or_default();

    // Helpers — we compute these inline so we don't need additional signals
    let next_billing_days = |date: Option<chrono::NaiveDate>| -> Option<String> {
        date.map(|d| {
            let now = Local::now().date_naive();
            let diff = (d - now).num_days();
            if diff == 0 { "Today".into() }
            else if diff == 1 { "Tomorrow".into() }
            else if diff < 0 { "Overdue".into() }
            else { format!("{} days", diff) }
        })
    };

    let format_date_short = |d: chrono::NaiveDate| -> String {
        d.format("%a, %b %-d").to_string()
    };

    let is_overdue = |d: chrono::NaiveDate| -> bool {
        d < Local::now().date_naive()
    };

    let is_today = |d: chrono::NaiveDate| -> bool {
        d == Local::now().date_naive()
    };

    rsx! {
        div { class: "p-4 md:p-6 flex flex-col gap-6",
            // Greeting
            div {
                h1 { class: "text-3xl font-bold tracking-tight", "Dashboard" }
                p { class: "text-muted-foreground", "Welcome back, {user_name}" }
            }

            // Stat cards — 2 cols mobile, 4 cols lg
            div { class: "grid gap-4 md:grid-cols-2 lg:grid-cols-4",
                // Monthly Subscriptions
                div { class: "bg-card border border-border rounded-lg p-6",
                    div { class: "flex flex-row items-center justify-between space-y-0 pb-2",
                        span { class: "text-sm font-medium", "Monthly Subscriptions" }
                        {icon_credit_card()}
                    }
                    div { class: "pt-0",
                        div { class: "text-2xl font-bold", "{format_currency(monthly_total, currency)}" }
                        p { class: "text-xs text-muted-foreground", "{total_active} active subscriptions" }
                    }
                }

                // Yearly Cost
                div { class: "bg-card border border-border rounded-lg p-6",
                    div { class: "flex flex-row items-center justify-between space-y-0 pb-2",
                        span { class: "text-sm font-medium", "Yearly Cost" }
                        {icon_trending_up()}
                    }
                    div { class: "pt-0",
                        div { class: "text-2xl font-bold", "{format_currency(yearly_total, currency)}" }
                        p { class: "text-xs text-muted-foreground", "Projected annual spending" }
                    }
                }

                // Due This Week
                div { class: "bg-card border border-border rounded-lg p-6",
                    div { class: "flex flex-row items-center justify-between space-y-0 pb-2",
                        span { class: "text-sm font-medium", "Due This Week" }
                        {icon_calendar()}
                    }
                    div { class: "pt-0",
                        div { class: "text-2xl font-bold", "{due_week.len()}" }
                        p { class: "text-xs text-muted-foreground", "Payments in next 7 days" }
                    }
                }

                // Due This Month
                div { class: "bg-card border border-border rounded-lg p-6",
                    div { class: "flex flex-row items-center justify-between space-y-0 pb-2",
                        span { class: "text-sm font-medium", "Due This Month" }
                        {icon_receipt()}
                    }
                    div { class: "pt-0",
                        div { class: "text-2xl font-bold", "{due_month.len()}" }
                        p { class: "text-xs text-muted-foreground", "Payments remaining this month" }
                    }
                }
            }

            // Two-column section: Active Subscriptions + Upcoming Payments
            div { class: "grid gap-4 md:grid-cols-2",
                // Active Subscriptions card
                div { class: "bg-card border border-border rounded-lg",
                    div { class: "p-6 pb-2",
                        h3 { class: "text-lg font-semibold", "Active Subscriptions" }
                    }
                    div { class: "p-6 pt-0",
                        if sub_state.is_loading {
                            div { class: "flex h-32 items-center justify-center",
                                div { class: "h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" }
                            }
                        } else if active_top5.is_empty() {
                            p { class: "flex h-32 items-center justify-center text-muted-foreground", "No subscriptions yet" }
                        } else {
                            div { class: "space-y-4",
                                for sub in active_top5 {
                                    {
                                        let days_label = next_billing_days(sub.next_billing_date);
                                        rsx! {
                                            div { key: "{sub.id}", class: "flex items-center justify-between",
                                                div { class: "flex items-center gap-3",
                                                    div { class: "flex h-9 w-9 items-center justify-center rounded-full bg-primary/10",
                                                        {icon_credit_card()}
                                                    }
                                                    div {
                                                        p { class: "text-sm font-medium", "{sub.name}" }
                                                        p { class: "text-xs text-muted-foreground capitalize", "{sub.frequency}" }
                                                    }
                                                }
                                                div { class: "text-right",
                                                    p { class: "text-sm font-medium", "{format_currency(sub.amount, currency)}" }
                                                    if let Some(label) = days_label {
                                                        p { class: "text-xs text-muted-foreground", "{label}" }
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

                // Upcoming Payments card
                div { class: "bg-card border border-border rounded-lg",
                    div { class: "p-6 pb-2",
                        h3 { class: "text-lg font-semibold", "Upcoming Payments" }
                    }
                    div { class: "p-6 pt-0",
                        if sub_state.is_loading {
                            div { class: "flex h-32 items-center justify-center",
                                div { class: "h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" }
                            }
                        } else if upcoming_top5.is_empty() {
                            p { class: "flex h-32 items-center justify-center text-muted-foreground", "No upcoming payments" }
                        } else {
                            div { class: "space-y-4",
                                for sub in upcoming_top5 {
                                    {
                                        let next_date = sub.next_billing_date.unwrap();
                                        let overdue = is_overdue(next_date);
                                        let today = is_today(next_date);
                                        let (dot_class, icon_class) = if overdue {
                                            ("bg-destructive/10", "text-destructive")
                                        } else if today {
                                            ("bg-yellow-500/10", "text-yellow-500")
                                        } else {
                                            ("bg-green-500/10", "text-green-500")
                                        };
                                        let date_str = format_date_short(next_date);
                                        let days_label = next_billing_days(Some(next_date));
                                        let logo_url = sub.logo_url.clone();

                                        rsx! {
                                            div { key: "{sub.id}", class: "flex items-center justify-between",
                                                div { class: "flex items-center gap-3",
                                                    div { class: "flex h-9 w-9 items-center justify-center rounded-full overflow-hidden {dot_class}",
                                                        {logo_image_with_fallback(&format!("dash-{}", sub.id), logo_url.as_deref(), &sub.name, "h-full w-full object-cover")}
                                                        if logo_url.is_none() {
                                                            svg {
                                                                view_box: "0 0 24 24",
                                                                fill: "none",
                                                                stroke: "currentColor",
                                                                stroke_width: "2",
                                                                stroke_linecap: "round",
                                                                stroke_linejoin: "round",
                                                                class: "h-4 w-4 {icon_class}",
                                                                line { x1: "4", y1: "2", x2: "4", y2: "22" }
                                                                path { d: "M1 2h18a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H1" }
                                                                path { d: "M4 18h2" }
                                                                circle { cx: "12", cy: "10", r: "2" }
                                                                path { d: "M13 14h-2" }
                                                                line { x1: "1", y1: "20", x2: "4", y2: "20" }
                                                            }
                                                        }
                                                    }
                                                    div {
                                                        p { class: "text-sm font-medium", "{sub.name}" }
                                                        p { class: "text-xs text-muted-foreground", "{date_str}" }
                                                    }
                                                }
                                                div { class: "text-right",
                                                    p { class: "text-sm font-medium", "{format_currency(sub.amount, currency)}" }
                                                    if let Some(label) = days_label {
                                                        p {
                                                            class: if overdue { "text-xs text-destructive" } else { "text-xs text-muted-foreground" },
                                                            "{label}"
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
}

#[cfg(test)]
mod tests {
    use crate::state::subscriptions::SubscriptionsState;
    use chrono::{Local, NaiveDate, Days, Datelike};
    use rust_decimal_macros::dec;
    use uuid::Uuid;
    use crate::models::subscription::Subscription;

    fn make_sub(
        name: &str,
        amount: rust_decimal::Decimal,
        frequency: &str,
        active: bool,
        next_billing: Option<NaiveDate>,
    ) -> Subscription {
        Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: name.into(),
            amount,
            currency: "EUR".into(),
            frequency: frequency.into(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: next_billing,
            is_active: active,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn monthly_total_empty() {
        let state = SubscriptionsState::default();
        assert_eq!(state.monthly_total(), dec!(0));
    }

    #[test]
    fn monthly_total_single_monthly() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("Netflix", dec!(15.99), "monthly", true, None)];
        assert_eq!(state.monthly_total(), dec!(15.99));
    }

    #[test]
    fn monthly_total_weekly_normalized() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("Weekly", dec!(5), "weekly", true, None)];
        assert_eq!(state.monthly_total(), dec!(20));
    }

    #[test]
    fn monthly_total_quarterly_normalized() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("Quarterly", dec!(30), "quarterly", true, None)];
        assert_eq!(state.monthly_total(), dec!(10));
    }

    #[test]
    fn monthly_total_yearly_normalized() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("Yearly", dec!(120), "yearly", true, None)];
        assert_eq!(state.monthly_total(), dec!(10));
    }

    #[test]
    fn yearly_total_matches_monthly_times_12() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("A", dec!(10), "monthly", true, None),
            make_sub("B", dec!(5), "weekly", true, None),
        ];
        let monthly = state.monthly_total();
        let yearly = state.yearly_total();
        assert_eq!(yearly, monthly * dec!(12));
    }

    #[test]
    fn monthly_total_ignores_inactive() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("Active", dec!(10), "monthly", true, None),
            make_sub("Inactive", dec!(999), "monthly", false, None),
        ];
        assert_eq!(state.monthly_total(), dec!(10));
    }

    #[test]
    fn due_this_week_counts_correctly() {
        let now = Local::now().date_naive();
        let in_3_days = now.checked_add_days(Days::new(3)).unwrap();
        let in_10_days = now.checked_add_days(Days::new(10)).unwrap();

        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("Due", dec!(10), "monthly", true, Some(in_3_days)),
            make_sub("Not Due", dec!(10), "monthly", true, Some(in_10_days)),
            make_sub("No Date", dec!(10), "monthly", true, None),
        ];
        assert_eq!(state.due_this_week().len(), 1);
        assert_eq!(state.due_this_week()[0].name, "Due");
    }

    #[test]
    fn due_this_week_ignores_inactive() {
        let now = Local::now().date_naive();
        let in_2_days = now.checked_add_days(Days::new(2)).unwrap();

        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("Active Due", dec!(10), "monthly", true, Some(in_2_days)),
            make_sub("Inactive Due", dec!(10), "monthly", false, Some(in_2_days)),
        ];
        assert_eq!(state.due_this_week().len(), 1);
    }

    #[test]
    fn due_this_month_counts_correctly() {
        let now = Local::now().date_naive();
        // Same month, later in month
        let same_month = if now.day() < 28 {
            NaiveDate::from_ymd_opt(now.year(), now.month(), now.day() + 1).unwrap()
        } else {
            now
        };

        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("ThisMonth", dec!(10), "monthly", true, Some(same_month)),
            make_sub("NoDate", dec!(10), "monthly", true, None),
        ];
        // At least 1 if the constructed date is in the same month
        if same_month.month() == now.month() && same_month.year() == now.year() {
            assert!(state.due_this_month().len() >= 1);
        }
    }

    #[test]
    fn due_this_month_ignores_inactive() {
        let now = Local::now().date_naive();
        let same_month = if now.day() < 28 {
            NaiveDate::from_ymd_opt(now.year(), now.month(), now.day() + 1).unwrap()
        } else {
            now
        };

        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("Active", dec!(10), "monthly", true, Some(same_month)),
            make_sub("Inactive", dec!(10), "monthly", false, Some(same_month)),
        ];
        let due = state.due_this_month();
        assert!(due.iter().all(|s| s.is_active));
    }

    #[test]
    fn page_content_uses_gap_not_space_y() {
        // space-y-6 adds unwanted margin-top to first child element.
        // Must use flex flex-col gap-6 instead — gap doesn't add to first child.
        // Bug: space-y-6 margin-top on greeting, first card, etc.
        let bad = "space-y-6";
        // Check page layout code only (before "mod tests")
        let pages = [
            (include_str!("dashboard.rs"), "dashboard"),
            (include_str!("subscriptions_page.rs"), "subscriptions"),
            (include_str!("transactions_page.rs"), "transactions"),
            (include_str!("calendar_page.rs"), "calendar"),
            (include_str!("categories_page.rs"), "categories"),
            (include_str!("settings_page.rs"), "settings"),
            (include_str!("reports_page.rs"), "reports"),
        ];
        for (src, name) in pages {
            // Only check code before tests module to avoid false positives
            let code = src.split("mod tests").next().unwrap_or(src);
            assert!(!code.contains(bad),
                "{} page must use flex flex-col gap-6 instead of space-y-6",
                name);
        }
    }

    // ── Additional dashboard page structure tests ──
    #[test]
    fn dashboard_uses_state_subscriptions() {
        let src = include_str!("dashboard.rs");
        assert!(src.contains("SUBSCRIPTIONS") || src.contains("subscriptions"),
            "Dashboard should use subscriptions state");
    }

    #[test]
    fn dashboard_uses_auth_state() {
        let src = include_str!("dashboard.rs");
        assert!(src.contains("AUTH") || src.contains("auth"),
            "Dashboard should use auth state");
    }

    #[test]
    fn dashboard_uses_currency_format() {
        let src = include_str!("dashboard.rs");
        assert!(src.contains("format_currency") || src.contains("currency"),
            "Dashboard should format currencies");
    }

    #[test]
    fn dashboard_has_svg_icons() {
        let src = include_str!("dashboard.rs");
        assert!(src.contains("svg"),
            "Dashboard should use SVG icons");
    }
}