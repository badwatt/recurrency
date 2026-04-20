use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTransactionRequest {
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub date: NaiveDate,
    pub category_id: Option<Uuid>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTransactionRequest {
    pub name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
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

impl From<Transaction> for TransactionResponse {
    fn from(t: Transaction) -> Self {
        Self {
            id: t.id,
            user_id: t.user_id,
            category_id: t.category_id,
            name: t.name,
            amount: t.amount,
            currency: t.currency,
            date: t.date,
            notes: t.notes,
            logo_url: t.logo_url,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_transaction_request_fields() {
        let req = CreateTransactionRequest {
            name: "Coffee".to_string(),
            amount: Decimal::from(450),
            currency: "EUR".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            category_id: None,
            notes: Some("Morning coffee".to_string()),
            logo_url: None,
        };
        assert_eq!(req.name, "Coffee");
        assert!(req.category_id.is_none());
    }

    #[test]
    fn update_transaction_all_none() {
        let req = UpdateTransactionRequest {
            name: None,
            amount: None,
            currency: None,
            date: None,
            category_id: None,
            notes: None,
            logo_url: None,
        };
        assert!(req.name.is_none());
        assert!(req.amount.is_none());
    }

    #[test]
    fn update_transaction_some_fields() {
        let req = UpdateTransactionRequest {
            name: Some("New Name".to_string()),
            amount: Some(Decimal::TEN),
            currency: None,
            date: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            category_id: None,
            notes: Some("Updated notes".to_string()),
            logo_url: None,
        };
        assert_eq!(req.name.as_deref(), Some("New Name"));
        assert_eq!(req.notes.as_deref(), Some("Updated notes"));
    }

    #[test]
    fn transaction_response_serialization() {
        let uid = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let cat_id = Uuid::new_v4();
        let resp = TransactionResponse {
            id: uid,
            user_id,
            category_id: cat_id,
            name: "Groceries".to_string(),
            amount: Decimal::from(5000),
            currency: "USD".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(),
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["name"], "Groceries");
        assert!(json.get("date").is_some());
    }

    #[test]
    fn transaction_from_transaction_response() {
        let uid = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let cat_id = Uuid::new_v4();
        let t = Transaction {
            id: uid,
            user_id,
            category_id: cat_id,
            name: "Coffee".to_string(),
            amount: Decimal::from(450),
            currency: "EUR".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            notes: Some("latte".to_string()),
            logo_url: Some("https://logo.com/c.png".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = TransactionResponse::from(t);
        assert_eq!(resp.id, uid);
        assert_eq!(resp.name, "Coffee");
        assert_eq!(resp.notes, Some("latte".to_string()));
    }

    #[test]
    fn create_transaction_request_missing_optional() {
        let req = CreateTransactionRequest {
            name: "Gas".to_string(),
            amount: Decimal::from(5000),
            currency: "USD".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            category_id: None,
            notes: None,
            logo_url: None,
        };
        assert!(req.category_id.is_none());
        assert!(req.notes.is_none());
    }

    #[test]
    fn transaction_with_logo_url() {
        let t = Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Amazon".to_string(),
            amount: Decimal::from(4999),
            currency: "USD".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            notes: Some("Prime membership".to_string()),
            logo_url: Some("https://logo.com/amazon.png".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(t.logo_url.is_some());
        assert!(t.notes.is_some());
    }

    #[test]
    fn transaction_without_optional_fields() {
        let t = Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Coffee".to_string(),
            amount: Decimal::from(450),
            currency: "EUR".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(t.notes.is_none());
        assert!(t.logo_url.is_none());
    }

    #[test]
    fn create_transaction_with_category_id() {
        let cat_id = Uuid::new_v4();
        let req = CreateTransactionRequest {
            name: "Dinner".to_string(),
            amount: Decimal::from(7500),
            currency: "USD".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            category_id: Some(cat_id),
            notes: Some("Restaurant".to_string()),
            logo_url: None,
        };
        assert_eq!(req.category_id, Some(cat_id));
        assert_eq!(req.notes.as_deref(), Some("Restaurant"));
    }

    #[test]
    fn update_transaction_with_logo_url() {
        let req = UpdateTransactionRequest {
            name: None,
            amount: None,
            currency: None,
            date: None,
            category_id: None,
            notes: None,
            logo_url: Some("https://logo.com/new.png".to_string()),
        };
        assert!(req.logo_url.is_some());
    }

    #[test]
    fn transaction_response_with_notes_and_logo() {
        let resp = TransactionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Grocery".to_string(),
            amount: Decimal::from(8500),
            currency: "GBP".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(),
            notes: Some("Weekly shopping".to_string()),
            logo_url: Some("https://logo.com/store.png".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["notes"], "Weekly shopping");
        assert!(json["logo_url"].is_string());
    }

    #[test]
    fn transaction_response_from_transaction_roundtrip() {
        let uid = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let cat_id = Uuid::new_v4();
        let t = Transaction {
            id: uid,
            user_id,
            category_id: cat_id,
            name: "Lunch".to_string(),
            amount: Decimal::from(1200),
            currency: "JPY".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 6, 20).unwrap(),
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = TransactionResponse::from(t);
        assert_eq!(resp.id, uid);
        assert_eq!(resp.name, "Lunch");
        assert_eq!(resp.currency, "JPY");
        assert_eq!(resp.category_id, cat_id);
        assert!(resp.notes.is_none());
        assert!(resp.logo_url.is_none());
    }
}
