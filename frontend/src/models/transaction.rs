use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub date: NaiveDate,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateTransactionRequest {
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub date: NaiveDate,
    pub category_id: Option<Uuid>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateTransactionRequest {
    pub name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn transaction_serialization_roundtrip() {
        let txn = Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Groceries".into(),
            amount: dec!(42.50),
            currency: "EUR".into(),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            notes: Some("Weekly shop".into()),
            logo_url: None,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&txn).unwrap();
        let back: Transaction = serde_json::from_str(&json).unwrap();
        assert_eq!(txn, back);
    }

    #[test]
    fn create_request_minimal() {
        let req = CreateTransactionRequest {
            name: "Coffee".into(),
            amount: dec!(3.50),
            currency: "EUR".into(),
            date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            category_id: None,
            notes: None,
            logo_url: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Coffee"));
    }
}