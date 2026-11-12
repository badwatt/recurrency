use crate::api::client::{api_delete, api_get, api_post, api_put, ApiError};
use crate::models::debt::{CreateDebtRequest, Debt, DebtPayment, RecordDebtPaymentRequest, UpdateDebtRequest};
use uuid::Uuid;

pub async fn list() -> Result<Vec<Debt>, ApiError> {
    api_get("/api/debts").await
}

pub async fn create(req: &CreateDebtRequest) -> Result<Debt, ApiError> {
    api_post("/api/debts", req).await
}

pub async fn update(id: &Uuid, req: &UpdateDebtRequest) -> Result<Debt, ApiError> {
    api_put(&format!("/api/debts/{id}"), req).await
}

pub async fn delete(id: &Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/debts/{id}")).await
}

pub async fn record_payment(id: &Uuid, req: &RecordDebtPaymentRequest) -> Result<Debt, ApiError> {
    api_post(&format!("/api/debts/{id}/payments"), req).await
}

pub async fn payments(id: &Uuid) -> Result<Vec<DebtPayment>, ApiError> {
    api_get(&format!("/api/debts/{id}/payments")).await
}
