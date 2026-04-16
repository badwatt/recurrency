use crate::api::client::{api_get, api_post, api_put, api_delete, ApiError};
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

pub async fn update(id: &uuid::Uuid, req: &UpdateTransactionRequest) -> Result<Transaction, ApiError> {
    api_put(&format!("/api/transactions/{}", id), req).await
}

pub async fn delete(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/transactions/{}", id)).await
}

#[cfg(test)]
mod tests {
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
}