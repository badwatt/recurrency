use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::accounts::models::Account;
use crate::categories::models::Category;
use crate::debts::models::{Debt, DebtPayment};
use crate::subscriptions::models::Subscription;
use crate::transactions::models::Transaction;

pub const BACKUP_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBackup {
    pub format_version: u32,
    pub app_version: String,
    pub exported_at: DateTime<Utc>,
    pub accounts: Vec<Account>,
    pub categories: Vec<Category>,
    pub subscriptions: Vec<Subscription>,
    pub transactions: Vec<Transaction>,
    pub debts: Vec<Debt>,
    pub debt_payments: Vec<DebtPayment>,
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub accounts: usize,
    pub categories: usize,
    pub subscriptions: usize,
    pub transactions: usize,
    pub debts: usize,
    pub debt_payments: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_format_is_versioned() {
        assert_eq!(BACKUP_FORMAT_VERSION, 1);
    }

    #[test]
    fn import_result_counts_are_serializable() {
        let result = ImportResult {
            accounts: 1,
            categories: 2,
            subscriptions: 3,
            transactions: 4,
            debts: 5,
            debt_payments: 6,
        };
        let json = serde_json::to_value(result).unwrap();
        assert_eq!(json["debts"], 5);
    }
}
