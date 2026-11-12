use std::collections::HashMap;

use dioxus::prelude::*;
use rust_decimal::Decimal;

use crate::models::currency::RatesResponse;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RatesState {
    pub base: String,
    pub provider: String,
    pub rates: HashMap<String, Decimal>,
}

impl RatesState {
    pub fn set_data(&mut self, data: RatesResponse) {
        self.base = data.base;
        self.provider = data.provider;
        self.rates = data.rates;
    }

    /// Convert `amount` from its currency into the user's main currency.
    /// Unknown rates deliberately fall back to the original amount rather than
    /// hiding the value.
    pub fn convert(&self, amount: Decimal, from: &str) -> Decimal {
        if self.base.is_empty() || from.eq_ignore_ascii_case(&self.base) {
            return amount;
        }
        let Some(rate) = self.rates.get(&from.to_uppercase()) else {
            return amount;
        };
        if *rate <= Decimal::ZERO {
            return amount;
        }
        amount / *rate
    }
}

pub static RATES: GlobalSignal<RatesState> = Signal::global(RatesState::default);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_into_base_currency() {
        let mut state = RatesState::default();
        state.base = "EUR".into();
        state.rates.insert("USD".into(), Decimal::new(11, 1));
        assert_eq!(state.convert(Decimal::from(11), "USD"), Decimal::from(10));
    }

    #[test]
    fn same_currency_is_identity() {
        let mut state = RatesState::default();
        state.base = "EUR".into();
        assert_eq!(state.convert(Decimal::from(10), "EUR"), Decimal::from(10));
    }

    #[test]
    fn missing_rate_keeps_amount() {
        let state = RatesState::default();
        assert_eq!(state.convert(Decimal::from(10), "GBP"), Decimal::from(10));
    }
}
