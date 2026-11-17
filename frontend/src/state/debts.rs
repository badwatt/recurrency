use crate::models::debt::Debt;
use dioxus::prelude::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DebtsState {
    pub debts: Vec<Debt>,
    pub is_loading: bool,
    pub error: Option<String>,
}

impl DebtsState {
    pub fn set_loading(&mut self) {
        self.is_loading = true;
        self.error = None;
    }

    pub fn set_data(&mut self, debts: Vec<Debt>) {
        self.debts = debts;
        self.is_loading = false;
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.is_loading = false;
    }

    pub fn active(&self) -> Vec<&Debt> {
        self.debts.iter().filter(|debt| debt.is_active).collect()
    }

    pub fn subscription_ids(&self) -> Vec<uuid::Uuid> {
        self.debts
            .iter()
            .filter_map(|debt| debt.subscription_id)
            .collect()
    }

    pub fn is_debt_subscription(&self, subscription_id: uuid::Uuid) -> bool {
        self.debts
            .iter()
            .any(|debt| debt.subscription_id == Some(subscription_id))
    }

    pub fn remaining_total(&self) -> rust_decimal::Decimal {
        self.active()
            .iter()
            .map(|debt| debt.remaining_amount)
            .sum()
    }
}

pub static DEBTS: GlobalSignal<DebtsState> = Signal::global(DebtsState::default);

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn remaining_total_ignores_paid() {
        let mut state = DebtsState::default();
        let mut debt = crate::models::debt::Debt {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            account_id: uuid::Uuid::nil(),
            category_id: uuid::Uuid::nil(),
            name: "Loan".into(),
            currency: "EUR".into(),
            total_amount: dec!(100),
            remaining_amount: dec!(40),
            paid_amount: dec!(60),
            debt_type: "manual".into(),
            monthly_payment: None,
            payment_days: Vec::new(),
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            next_payment_date: None,
            subscription_id: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        state.debts.push(debt.clone());
        debt.is_active = false;
        debt.remaining_amount = dec!(0);
        state.debts.push(debt);
        assert_eq!(state.remaining_total(), dec!(40));
    }

    fn sample_debt(active: bool, subscription_id: Option<uuid::Uuid>) -> Debt {
        Debt {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::nil(),
            account_id: uuid::Uuid::nil(),
            category_id: uuid::Uuid::nil(),
            name: "Loan".into(),
            currency: "EUR".into(),
            total_amount: dec!(100),
            remaining_amount: dec!(50),
            paid_amount: dec!(50),
            debt_type: "manual".into(),
            monthly_payment: None,
            payment_days: Vec::new(),
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            next_payment_date: None,
            subscription_id,
            is_active: active,
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn loading_data_and_errors_replace_each_other() {
        let mut state = DebtsState::default();
        state.set_loading();
        assert!(state.is_loading);
        assert!(state.error.is_none());

        state.set_error("offline".into());
        assert!(!state.is_loading);
        assert_eq!(state.error.as_deref(), Some("offline"));

        state.set_data(vec![sample_debt(true, None)]);
        assert!(!state.is_loading);
        assert!(state.error.is_none());
        assert_eq!(state.debts.len(), 1);
    }

    #[test]
    fn active_and_subscription_helpers_filter_debts() {
        let subscription = uuid::Uuid::new_v4();
        let mut state = DebtsState {
            debts: vec![
                sample_debt(true, Some(subscription)),
                sample_debt(false, None),
            ],
            ..Default::default()
        };
        assert_eq!(state.active().len(), 1);
        assert_eq!(state.subscription_ids(), vec![subscription]);
        assert!(state.is_debt_subscription(subscription));
        assert!(!state.is_debt_subscription(uuid::Uuid::new_v4()));

        state.debts.clear();
        assert_eq!(state.remaining_total(), dec!(0));
        assert!(state.active().is_empty());
    }
}
