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
}
