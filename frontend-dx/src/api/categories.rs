use crate::api::client::{api_get, api_post, api_put, api_delete, ApiError};
use crate::models::category::*;

pub async fn list() -> Result<Vec<Category>, ApiError> {
    api_get("/api/categories").await
}

pub async fn create(req: &CreateCategoryRequest) -> Result<Category, ApiError> {
    api_post("/api/categories", req).await
}

pub async fn update(id: &uuid::Uuid, req: &UpdateCategoryRequest) -> Result<Category, ApiError> {
    api_put(&format!("/api/categories/{}", id), req).await
}

pub async fn delete(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/categories/{}", id)).await
}

pub async fn set_default(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_put("/api/categories/default", &serde_json::json!({ "id": id })).await
}