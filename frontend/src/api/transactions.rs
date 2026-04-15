use crate::api::client::{api_get, api_post, api_put, api_delete, ApiError};
use crate::models::transaction::*;

pub async fn list() -> Result<Vec<Transaction>, ApiError> {
    api_get("/api/transactions").await
}

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