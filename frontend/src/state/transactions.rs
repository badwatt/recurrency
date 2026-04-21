use crate::models::transaction::Transaction;
use chrono::Datelike;
use dioxus::prelude::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TransactionsState {
    pub transactions: Vec<Transaction>,
    pub is_loading: bool,
    pub error: Option<String>,
}

impl TransactionsState {
    /// Group transactions by year-month (descending).
    pub fn grouped_by_month(&self) -> Vec<(String, Vec<&Transaction>)> {
        let mut groups: std::collections::BTreeMap<String, Vec<&Transaction>> =
            std::collections::BTreeMap::new();

        for txn in &self.transactions {
            let key = format!("{}-{:02}", txn.date.year(), txn.date.month());
            groups.entry(key).or_default().push(txn);
        }

        // Sort descending
        let mut result: Vec<_> = groups.into_iter().collect();
        result.sort_by(|a, b| b.0.cmp(&a.0));
        result
    }

    /// Set loading.
    pub fn set_loading(&mut self) {
        self.is_loading = true;
        self.error = None;
    }

    /// Set data.
    pub fn set_data(&mut self, txns: Vec<Transaction>) {
        self.transactions = txns;
        self.is_loading = false;
        self.error = None;
    }

    /// Set error.
    pub fn set_error(&mut self, err: String) {
        self.error = Some(err);
        self.is_loading = false;
    }
}

pub static TRANSACTIONS: GlobalSignal<TransactionsState> =
    Signal::global(|| TransactionsState::default());

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn make_txn(name: &str, date: NaiveDate) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: name.into(),
            amount: dec!(10),
            currency: "EUR".into(),
            date,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn default_empty() {
        let state = TransactionsState::default();
        assert!(state.transactions.is_empty());
    }

    #[test]
    fn grouped_by_month_descending() {
        let mut state = TransactionsState::default();
        state.transactions = vec![
            make_txn("Jan", NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            make_txn("Mar", NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
            make_txn("Jan2", NaiveDate::from_ymd_opt(2024, 1, 20).unwrap()),
            make_txn("Feb", NaiveDate::from_ymd_opt(2024, 2, 10).unwrap()),
        ];

        let groups = state.grouped_by_month();
        assert_eq!(groups[0].0, "2024-03");
        assert_eq!(groups[0].1.len(), 1);
        assert_eq!(groups[1].0, "2024-02");
        assert_eq!(groups[2].0, "2024-01");
        assert_eq!(groups[2].1.len(), 2);
    }

    #[test]
    fn set_data_and_error() {
        let mut state = TransactionsState::default();
        state.set_loading();
        assert!(state.is_loading);

        state.set_data(vec![make_txn(
            "Test",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )]);
        assert!(!state.is_loading);
        assert_eq!(state.transactions.len(), 1);

        state.set_error("fail".into());
        assert!(!state.is_loading);
        assert_eq!(state.error, Some("fail".into()));
    }

    #[test]
    fn set_loading_clears_error() {
        let mut state = TransactionsState::default();
        state.set_error("error".into());
        state.set_loading();
        assert!(state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn set_data_clears_error() {
        let mut state = TransactionsState::default();
        state.set_error("error".into());
        state.set_data(vec![make_txn(
            "A",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )]);
        assert!(state.error.is_none());
        assert!(!state.is_loading);
    }

    #[test]
    fn set_error_clears_loading() {
        let mut state = TransactionsState::default();
        state.set_loading();
        state.set_error("fail".into());
        assert!(!state.is_loading);
    }

    #[test]
    fn set_data_replaces_transactions() {
        let mut state = TransactionsState::default();
        state.transactions = vec![make_txn(
            "Old",
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        )];
        state.set_data(vec![make_txn(
            "New",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )]);
        assert_eq!(state.transactions.len(), 1);
        assert_eq!(state.transactions[0].name, "New");
    }

    #[test]
    fn grouped_by_month_empty() {
        let state = TransactionsState::default();
        let groups = state.grouped_by_month();
        assert!(groups.is_empty());
    }

    #[test]
    fn grouped_by_month_single_month() {
        let mut state = TransactionsState::default();
        state.transactions = vec![
            make_txn("A", NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
            make_txn("B", NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
        ];
        let groups = state.grouped_by_month();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, "2024-03");
        assert_eq!(groups[0].1.len(), 2);
    }

    #[test]
    fn grouped_by_month_cross_year() {
        let mut state = TransactionsState::default();
        state.transactions = vec![
            make_txn("Jan24", NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            make_txn("Dec23", NaiveDate::from_ymd_opt(2023, 12, 1).unwrap()),
        ];
        let groups = state.grouped_by_month();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, "2024-01"); // Most recent first
        assert_eq!(groups[1].0, "2023-12");
    }

    #[test]
    fn set_error_after_loading_clears_it() {
        let mut state = TransactionsState::default();
        state.set_loading();
        state.set_error("error".into());
        assert!(!state.is_loading);
    }

    #[test]
    fn set_data_with_empty_vec() {
        let mut state = TransactionsState::default();
        state.set_loading();
        state.set_data(vec![]);
        assert!(!state.is_loading);
        assert!(state.transactions.is_empty());
        assert!(state.error.is_none());
    }

    // ── Additional TransactionsState tests ──
    #[test]
    fn transactions_state_default_values() {
        let state = TransactionsState::default();
        assert!(!state.is_loading);
        assert!(state.error.is_none());
        assert!(state.transactions.is_empty());
    }

    #[test]
    fn transactions_state_clone() {
        let state = TransactionsState::default();
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn transactions_state_debug() {
        let state = TransactionsState::default();
        let debug = format!("{:?}", state);
        assert!(debug.contains("TransactionsState"));
    }

    #[test]
    fn grouped_by_month_same_month_different_days() {
        let mut state = TransactionsState::default();
        state.transactions = vec![
            make_txn("A", NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
            make_txn("B", NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            make_txn("C", NaiveDate::from_ymd_opt(2024, 6, 30).unwrap()),
        ];
        let groups = state.grouped_by_month();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].1.len(), 3);
    }

    #[test]
    fn set_error_replaces_previous_error() {
        let mut state = TransactionsState::default();
        state.set_error("first error".into());
        assert_eq!(state.error, Some("first error".into()));
        state.set_error("second error".into());
        assert_eq!(state.error, Some("second error".into()));
    }

    #[test]
    fn set_loading_then_data_then_error() {
        let mut state = TransactionsState::default();
        state.set_loading();
        assert!(state.is_loading);
        state.set_data(vec![make_txn(
            "A",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )]);
        assert!(!state.is_loading);
        state.set_error("fail".into());
        assert!(!state.is_loading);
        assert_eq!(state.error, Some("fail".into()));
    }
}
