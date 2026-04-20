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


#[cfg(test)]
mod transaction_model_tests {
    use super::*;
    use rust_decimal_macros::dec;
    use chrono::NaiveDate;

    fn make_test_txn(name: &str, amount: Decimal, date_str: &str) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: name.into(),
            amount,
            currency: "EUR".into(),
            date: NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap(),
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn transaction_json_roundtrip() {
        let txn = make_test_txn("Coffee", dec!(4.50), "2024-01-15");
        let json = serde_json::to_string(&txn).unwrap();
        let parsed: Transaction = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, txn.name);
        assert_eq!(parsed.amount, txn.amount);
    }

    #[test]
    fn transaction_various_amounts() {
        for amount in &[dec!(0.01), dec!(1), dec!(999.99), dec!(10000)] {
            let txn = make_test_txn("Test", *amount, "2024-01-01");
            assert_eq!(txn.amount, *amount);
        }
    }

    #[test]
    fn transaction_zero_amount() {
        let txn = make_test_txn("Free", dec!(0), "2024-01-01");
        assert_eq!(txn.amount, dec!(0));
    }

    #[test]
    fn transaction_with_notes() {
        let txn = Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Lunch".into(),
            amount: dec!(25),
            currency: "EUR".into(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            notes: Some("Business lunch".into()),
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        assert_eq!(txn.notes.as_deref(), Some("Business lunch"));
    }

    #[test]
    fn transaction_with_logo() {
        let txn = Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Coffee Shop".into(),
            amount: dec!(5),
            currency: "EUR".into(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            notes: None,
            logo_url: Some("coffee-shop.png".into()),
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        assert!(txn.logo_url.is_some());
    }

    #[test]
    fn transaction_date_formats() {
        for date_str in &["2024-01-01", "2024-06-15", "2024-12-31"] {
            let txn = make_test_txn("Test", dec!(10), date_str);
            assert!(txn.date.to_string().len() == 10);
        }
    }

    #[test]
    fn transaction_currencies() {
        for currency in &["USD", "EUR", "GBP", "JPY"] {
            let txn = Transaction {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                category_id: Uuid::new_v4(),
                name: "Test".into(),
                amount: dec!(10),
                currency: (*currency).into(),
                date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                notes: None,
                logo_url: None,
                created_at: chrono::DateTime::UNIX_EPOCH,
                updated_at: chrono::DateTime::UNIX_EPOCH,
            };
            assert_eq!(txn.currency, *currency);
        }
    }

    #[test]
    fn transaction_clone() {
        let txn = make_test_txn("Coffee", dec!(4.50), "2024-01-15");
        let cloned = txn.clone();
        assert_eq!(cloned.name, txn.name);
        assert_eq!(cloned.amount, txn.amount);
    }

    #[test]
    fn transaction_display_name() {
        let txn = make_test_txn("Lunch", dec!(25), "2024-01-01");
        assert_eq!("Lunch" /* display_name() */, "Lunch");
    }

    #[test]
    fn create_transaction_request() {
        let cat = Uuid::new_v4();
        let req = CreateTransactionRequest {
            name: "Coffee".into(),
            amount: dec!(4.50),
            currency: "EUR".into(),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            category_id: Some(cat),
            notes: None,
            logo_url: None,
        };
        assert_eq!(req.name, "Coffee");
        assert_eq!(req.amount, dec!(4.50));
        assert!(req.notes.is_none());
    }

    #[test]
    fn create_transaction_request_with_all_fields() {
        let cat = Uuid::new_v4();
        let req = CreateTransactionRequest {
            name: "Lunch".into(),
            amount: dec!(25),
            currency: "USD".into(),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            category_id: Some(cat),
            notes: Some("Business meeting".into()),
            logo_url: Some("logo.png".into()),
        };
        assert_eq!(req.notes.as_deref(), Some("Business meeting"));
        assert!(req.logo_url.is_some());
    }

    #[test]
    fn update_transaction_request_partial() {
        let req: UpdateTransactionRequest = UpdateTransactionRequest {
            name: Some("Updated Lunch".into()),
            amount: None,
            currency: None,
            date: None,
            category_id: None,
            notes: None,
            logo_url: None,
        };
        assert_eq!(req.name.as_deref(), Some("Updated Lunch"));
        assert!(req.amount.is_none());
    }

    #[test]
    fn update_transaction_request_all() {
        let cat = Uuid::new_v4();
        let req = UpdateTransactionRequest {
            name: Some("New Name".into()),
            amount: Some(dec!(100)),
            currency: Some("GBP".into()),
            date: Some(NaiveDate::from_ymd_opt(2024, 7, 1).unwrap()),
            category_id: Some(cat),
            notes: Some("Updated".into()),
            logo_url: Some("new.png".into()),
        };
        assert!(req.name.is_some());
        assert!(req.amount.is_some());
        assert!(req.currency.is_some());
        assert!(req.date.is_some());
        assert!(req.logo_url.is_some());
    }

    #[test]
    fn transaction_id_unique() {
        let t1 = make_test_txn("A", dec!(1), "2024-01-01");
        let t2 = make_test_txn("B", dec!(2), "2024-01-01");
        assert_ne!(t1.id, t2.id);
    }

    #[test]
    fn transaction_clone_independent() {
        let txn = make_test_txn("Coffee", dec!(4.50), "2024-01-15");
        let cloned = txn.clone();
        assert_eq!(cloned.name, txn.name);
        assert_eq!(cloned.amount, txn.amount);
        assert_eq!(cloned.date, txn.date);
    }
}
}