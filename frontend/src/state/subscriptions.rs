use crate::models::subscription::Subscription;
use dioxus::prelude::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubscriptionsState {
    pub subscriptions: Vec<Subscription>,
    pub is_loading: bool,
    pub error: Option<String>,
}

impl SubscriptionsState {
    /// Get only active subscriptions.
    pub fn active(&self) -> Vec<&Subscription> {
        self.subscriptions.iter().filter(|s| s.is_active).collect()
    }

    /// Calculate monthly total from active subscriptions.
    /// Normalizes weekly/quarterly/yearly to monthly equivalent.
    pub fn monthly_total(&self) -> rust_decimal::Decimal {
        self.active().iter().map(|s| monthly_normalized(s)).sum()
    }

    /// Calculate yearly total from active subscriptions.
    pub fn yearly_total(&self) -> rust_decimal::Decimal {
        self.active().iter().map(|s| yearly_normalized(s)).sum()
    }

    /// Get subscriptions due in the next 7 days.
    pub fn due_this_week(&self) -> Vec<&Subscription> {
        use chrono::{Days, Local};
        let now = Local::now().date_naive();
        let week = now.checked_add_days(Days::new(7)).unwrap();
        self.active()
            .into_iter()
            .filter(|s| {
                s.next_billing_date
                    .map(|d| d >= now && d <= week)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get subscriptions due in the current month.
    pub fn due_this_month(&self) -> Vec<&Subscription> {
        use chrono::{Datelike, Local};
        let now = Local::now().date_naive();
        self.active()
            .into_iter()
            .filter(|s| {
                s.next_billing_date
                    .map(|d| d.month() == now.month() && d.year() == now.year())
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Set loading state.
    pub fn set_loading(&mut self) {
        self.is_loading = true;
        self.error = None;
    }

    /// Set data after successful fetch.
    pub fn set_data(&mut self, subs: Vec<Subscription>) {
        self.subscriptions = subs;
        self.is_loading = false;
        self.error = None;
    }

    /// Set error.
    pub fn set_error(&mut self, err: String) {
        self.error = Some(err);
        self.is_loading = false;
    }
}

/// Normalize subscription amount to monthly equivalent.
fn monthly_normalized(sub: &Subscription) -> rust_decimal::Decimal {
    use rust_decimal::Decimal;
    match sub.frequency.as_str() {
        "weekly" => sub.amount * Decimal::from(4),
        "monthly" => sub.amount,
        "quarterly" => sub.amount / Decimal::from(3),
        "yearly" => sub.amount / Decimal::from(12),
        _ => sub.amount,
    }
}

/// Normalize subscription amount to yearly equivalent.
fn yearly_normalized(sub: &Subscription) -> rust_decimal::Decimal {
    monthly_normalized(sub) * rust_decimal::Decimal::from(12)
}

pub static SUBSCRIPTIONS: GlobalSignal<SubscriptionsState> =
    Signal::global(|| SubscriptionsState::default());

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn make_sub(
        name: &str,
        amount: Decimal,
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
    fn default_state_empty() {
        let state = SubscriptionsState::default();
        assert!(state.subscriptions.is_empty());
        assert!(!state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn active_filters_inactive() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("Netflix", dec!(15), "monthly", true, None),
            make_sub("Old Sub", dec!(10), "monthly", false, None),
        ];
        assert_eq!(state.active().len(), 1);
        assert_eq!(state.active()[0].name, "Netflix");
    }

    #[test]
    fn monthly_total_mixed_frequencies() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("A", dec!(10), "monthly", true, None), // 10/mo
            make_sub("B", dec!(5), "weekly", true, None),   // 20/mo (5*4)
            make_sub("C", dec!(30), "quarterly", true, None), // 10/mo (30/3)
            make_sub("D", dec!(120), "yearly", true, None), // 10/mo (120/12)
        ];
        assert_eq!(state.monthly_total(), dec!(50));
    }

    #[test]
    fn yearly_total() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("A", dec!(10), "monthly", true, None)];
        assert_eq!(state.yearly_total(), dec!(120));
    }

    #[test]
    fn monthly_total_ignores_inactive() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("Active", dec!(10), "monthly", true, None),
            make_sub("Inactive", dec!(100), "monthly", false, None),
        ];
        assert_eq!(state.monthly_total(), dec!(10));
    }

    #[test]
    fn set_loading_state() {
        let mut state = SubscriptionsState::default();
        state.set_loading();
        assert!(state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn set_data_clears_loading() {
        let mut state = SubscriptionsState::default();
        state.set_loading();
        state.set_data(vec![make_sub("Test", dec!(10), "monthly", true, None)]);
        assert!(!state.is_loading);
        assert!(state.error.is_none());
        assert_eq!(state.subscriptions.len(), 1);
    }

    #[test]
    fn set_error_state() {
        let mut state = SubscriptionsState::default();
        state.set_loading();
        state.set_error("Network error".into());
        assert!(!state.is_loading);
        assert_eq!(state.error, Some("Network error".into()));
    }

    #[test]
    fn due_this_week() {
        use chrono::{Days, Local};
        let now = Local::now().date_naive();
        let in_3_days = now.checked_add_days(Days::new(3)).unwrap();
        let in_10_days = now.checked_add_days(Days::new(10)).unwrap();

        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("Due", dec!(10), "monthly", true, Some(in_3_days)),
            make_sub("Not Due", dec!(10), "monthly", true, Some(in_10_days)),
            make_sub("No Date", dec!(10), "monthly", true, None),
        ];

        let due = state.due_this_week();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].name, "Due");
    }

    #[test]
    fn due_this_month() {
        use chrono::{Days, Local};
        let now = Local::now().date_naive();
        let this_month = now;
        let next_month = now.checked_add_days(Days::new(32)).unwrap();

        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("ThisMonth", dec!(10), "monthly", true, Some(this_month)),
            make_sub("NextMonth", dec!(10), "monthly", true, Some(next_month)),
            make_sub("NoDate", dec!(10), "monthly", true, None),
        ];

        let due = state.due_this_month();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].name, "ThisMonth");
    }

    #[test]
    fn due_this_month_ignores_inactive() {
        use chrono::Local;
        let now = Local::now().date_naive();

        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("Active", dec!(10), "monthly", true, Some(now)),
            make_sub("Inactive", dec!(10), "monthly", false, Some(now)),
        ];

        let due = state.due_this_month();
        assert_eq!(due.len(), 1);
    }

    #[test]
    fn monthly_total_weekly_frequency() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("Weekly", dec!(5), "weekly", true, None)];
        assert_eq!(state.monthly_total(), dec!(20));
    }

    #[test]
    fn monthly_total_quarterly_frequency() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("Quarterly", dec!(30), "quarterly", true, None)];
        assert_eq!(state.monthly_total(), dec!(10));
    }

    #[test]
    fn monthly_total_yearly_frequency() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("Yearly", dec!(120), "yearly", true, None)];
        assert_eq!(state.monthly_total(), dec!(10));
    }

    #[test]
    fn monthly_total_empty() {
        let state = SubscriptionsState::default();
        assert_eq!(state.monthly_total(), dec!(0));
    }

    #[test]
    fn yearly_total_empty() {
        let state = SubscriptionsState::default();
        assert_eq!(state.yearly_total(), dec!(0));
    }

    #[test]
    fn set_data_replaces_subscriptions() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("Old", dec!(5), "monthly", true, None)];
        state.set_data(vec![make_sub("New", dec!(10), "monthly", true, None)]);
        assert_eq!(state.subscriptions.len(), 1);
        assert_eq!(state.subscriptions[0].name, "New");
    }

    #[test]
    fn set_error_clears_error_then_sets_new() {
        let mut state = SubscriptionsState::default();
        state.set_error("First error".into());
        assert_eq!(state.error, Some("First error".into()));
        state.set_error("Second error".into());
        assert_eq!(state.error, Some("Second error".into()));
    }

    #[test]
    fn set_loading_clears_error() {
        let mut state = SubscriptionsState::default();
        state.set_error("Some error".into());
        state.set_loading();
        assert!(state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn monthly_total_unknown_frequency() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("Custom", dec!(10), "biweekly", true, None)];
        // Unknown frequency falls through to just use the amount as-is
        assert_eq!(state.monthly_total(), dec!(10));
    }

    #[test]
    fn yearly_total_mixed_frequencies() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("A", dec!(10), "monthly", true, None), // 10*12 = 120
            make_sub("B", dec!(5), "weekly", true, None),   // 20*12 = 240
        ];
        assert_eq!(state.yearly_total(), dec!(360));
    }

    #[test]
    fn active_empty_when_all_inactive() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("A", dec!(10), "monthly", false, None),
            make_sub("B", dec!(20), "monthly", false, None),
        ];
        assert!(state.active().is_empty());
    }

    #[test]
    fn due_this_week_none_due() {
        use chrono::{Days, Local};
        let now = Local::now().date_naive();
        let in_30_days = now.checked_add_days(Days::new(30)).unwrap();

        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("Far", dec!(10), "monthly", true, Some(in_30_days))];
        let due = state.due_this_week();
        assert!(due.is_empty());
    }

    #[test]
    fn due_this_month_no_dates_set() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("NoDate", dec!(10), "monthly", true, None)];
        let due = state.due_this_month();
        assert!(due.is_empty());
    }

    // ── Additional SubscriptionsState tests ──
    #[test]
    fn subscriptions_state_default_values() {
        let state = SubscriptionsState::default();
        assert!(!state.is_loading);
        assert!(state.error.is_none());
        assert!(state.subscriptions.is_empty());
    }

    #[test]
    fn subscriptions_state_clone() {
        let state = SubscriptionsState::default();
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn subscriptions_state_debug() {
        let state = SubscriptionsState::default();
        let debug = format!("{:?}", state);
        assert!(debug.contains("SubscriptionsState"));
    }

    #[test]
    fn monthly_total_with_no_active() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("A", dec!(10), "monthly", false, None),
            make_sub("B", dec!(20), "monthly", false, None),
        ];
        assert_eq!(state.monthly_total(), dec!(0));
    }

    #[test]
    fn active_returns_correct_count() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![
            make_sub("A", dec!(10), "monthly", true, None),
            make_sub("B", dec!(20), "monthly", false, None),
            make_sub("C", dec!(30), "monthly", true, None),
        ];
        assert_eq!(state.active().len(), 2);
    }

    #[test]
    fn set_error_replaces_previous() {
        let mut state = SubscriptionsState::default();
        state.set_error("first".into());
        state.set_error("second".into());
        assert_eq!(state.error, Some("second".into()));
    }

    #[test]
    fn set_data_then_set_error() {
        let mut state = SubscriptionsState::default();
        state.set_data(vec![make_sub("A", dec!(10), "monthly", true, None)]);
        assert!(!state.is_loading);
        assert!(state.error.is_none());
        state.set_error("fail".into());
        assert!(!state.is_loading);
        assert_eq!(state.error, Some("fail".into()));
    }

    #[test]
    fn yearly_total_with_no_active() {
        let mut state = SubscriptionsState::default();
        state.subscriptions = vec![make_sub("A", dec!(10), "monthly", false, None)];
        assert_eq!(state.yearly_total(), dec!(0));
    }
}
