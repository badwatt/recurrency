use crate::api::client::{api_delete, api_get, api_post, api_put, ApiError};
use crate::models::transaction::*;

pub async fn list() -> Result<Vec<Transaction>, ApiError> {
    api_get("/api/transactions").await
}

#[allow(dead_code)]
pub async fn get(id: &uuid::Uuid) -> Result<Transaction, ApiError> {
    api_get(&format!("/api/transactions/{}", id)).await
}

pub async fn create(req: &CreateTransactionRequest) -> Result<Transaction, ApiError> {
    api_post("/api/transactions", req).await
}

pub async fn update(
    id: &uuid::Uuid,
    req: &UpdateTransactionRequest,
) -> Result<Transaction, ApiError> {
    api_put(&format!("/api/transactions/{}", id), req).await
}

pub async fn delete(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/transactions/{}", id)).await
}

#[cfg(test)]
mod tests {
    use crate::models::transaction::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    #[test]
    fn transaction_api_paths() {
        let base = "/api/transactions";
        assert!(base.starts_with("/api/"));

        let id = uuid::Uuid::new_v4();
        let detail = format!("/api/transactions/{}", id);
        assert!(detail.starts_with("/api/transactions/"));
    }

    #[test]
    fn transaction_id_in_path() {
        let id = uuid::Uuid::new_v4();
        let path = format!("/api/transactions/{}", id);
        assert!(path.contains(&id.to_string()));
    }

    #[test]
    fn transaction_list_path() {
        assert_eq!("/api/transactions", "/api/transactions");
    }

    #[test]
    fn transaction_model_fields() {
        let txn = Transaction {
            account_id: Uuid::nil(),
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Coffee".into(),
            amount: dec!(4.50),
            currency: "EUR".into(),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        assert_eq!(txn.name, "Coffee");
        assert_eq!(txn.amount, dec!(4.50));
        assert_eq!(txn.currency, "EUR");
        assert!(txn.notes.is_none());
    }

    #[test]
    fn transaction_with_notes() {
        let txn = Transaction {
            account_id: Uuid::nil(),
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Dinner".into(),
            amount: dec!(25.00),
            currency: "USD".into(),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            notes: Some("Birthday dinner".into()),
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        assert!(txn.notes.is_some());
        assert_eq!(txn.notes.as_ref().unwrap(), "Birthday dinner");
    }

    #[test]
    fn create_transaction_request_required_fields() {
        let req = CreateTransactionRequest {
            account_id: Uuid::nil(),
            name: "Lunch".into(),
            amount: dec!(12.50),
            currency: "EUR".into(),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            category_id: Some(Uuid::new_v4()),
            notes: None,
            logo_url: None,
        };
        assert_eq!(req.name, "Lunch");
        assert_eq!(req.amount, dec!(12.50));
        assert_eq!(req.date, NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    }

    #[test]
    fn create_transaction_request_with_notes() {
        let req = CreateTransactionRequest {
            account_id: Uuid::nil(),
            name: "Book".into(),
            amount: dec!(19.99),
            currency: "USD".into(),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            category_id: None,
            notes: Some("Programming book".into()),
            logo_url: None,
        };
        assert!(req.notes.is_some());
    }

    #[test]
    fn update_transaction_request_partial() {
        let req = UpdateTransactionRequest {
            account_id: None,
            name: Some("Updated".into()),
            amount: Some(dec!(30.00)),
            currency: None,
            date: None,
            category_id: None,
            notes: None,
            logo_url: None,
        };
        assert_eq!(req.name.as_deref(), Some("Updated"));
        assert_eq!(req.amount.as_ref().unwrap(), &dec!(30.00));
        assert!(req.currency.is_none());
    }

    #[test]
    fn update_transaction_request_all_none() {
        let req: UpdateTransactionRequest = Default::default();
        assert!(req.name.is_none());
        assert!(req.amount.is_none());
        assert!(req.currency.is_none());
        assert!(req.date.is_none());
        assert!(req.logo_url.is_none());
    }

    #[test]
    fn transaction_date_string() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(date.to_string(), "2024-06-15");
    }

    #[test]
    fn transaction_serialization_roundtrip() {
        let txn = Transaction {
            account_id: Uuid::nil(),
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Test".into(),
            amount: dec!(10.0),
            currency: "EUR".into(),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&txn).unwrap();
        let back: Transaction = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Test");
    }

    #[test]
    fn create_request_serialization() {
        let req = CreateTransactionRequest {
            account_id: Uuid::nil(),
            name: "Coffee".into(),
            amount: dec!(4.5),
            currency: "EUR".into(),
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            category_id: None,
            notes: None,
            logo_url: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Coffee"));
        assert!(json.contains("4.5"));
    }

    #[test]
    fn transaction_uuid_uniqueness() {
        let ids: std::collections::HashSet<_> = (0..100).map(|_| Uuid::new_v4()).collect();
        assert_eq!(ids.len(), 100);
    }
}
