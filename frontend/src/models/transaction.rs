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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
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

    #[test]
    fn update_request_default_all_none() {
        let req = UpdateTransactionRequest::default();
        assert!(req.name.is_none());
        assert!(req.amount.is_none());
        assert!(req.currency.is_none());
        assert!(req.date.is_none());
        assert!(req.category_id.is_none());
        assert!(req.notes.is_none());
        assert!(req.logo_url.is_none());
    }

    #[test]
    fn update_request_partial() {
        let req = UpdateTransactionRequest {
            name: Some("Updated".into()),
            amount: Some(dec!(99.99)),
            ..Default::default()
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdateTransactionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, Some("Updated".into()));
        assert_eq!(back.amount, Some(dec!(99.99)));
        assert!(back.date.is_none());
    }

    #[test]
    fn transaction_with_notes() {
        let txn = Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Groceries".into(),
            amount: dec!(42.50),
            currency: "EUR".into(),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            notes: Some("Weekly shop".into()),
            logo_url: Some("https://example.com/logo.png".into()),
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&txn).unwrap();
        let back: Transaction = serde_json::from_str(&json).unwrap();
        assert_eq!(back.notes.as_deref(), Some("Weekly shop"));
        assert_eq!(back.logo_url.as_deref(), Some("https://example.com/logo.png"));
    }

    #[test]
    fn create_request_with_category() {
        let req = CreateTransactionRequest {
            name: "Dinner".into(),
            amount: dec!(25.00),
            currency: "USD".into(),
            date: NaiveDate::from_ymd_opt(2024, 5, 10).unwrap(),
            category_id: Some(Uuid::new_v4()),
            notes: Some("Restaurant".into()),
            logo_url: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: CreateTransactionRequest = serde_json::from_str(&json).unwrap();
        assert!(back.category_id.is_some());
    }
}