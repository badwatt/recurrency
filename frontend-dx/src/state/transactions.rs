use chrono::Datelike;
use dioxus::prelude::*;
use crate::models::transaction::Transaction;

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

pub static TRANSACTIONS: GlobalSignal<TransactionsState> = Signal::global(|| TransactionsState::default());

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

        state.set_data(vec![make_txn("Test", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())]);
        assert!(!state.is_loading);
        assert_eq!(state.transactions.len(), 1);

        state.set_error("fail".into());
        assert!(!state.is_loading);
        assert_eq!(state.error, Some("fail".into()));
    }
}