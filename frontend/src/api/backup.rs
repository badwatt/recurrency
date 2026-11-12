use serde_json::Value;

use crate::api::client::{ApiError, api_get, api_post};

pub async fn export() -> Result<Value, ApiError> {
    api_get("/api/backup").await
}

pub async fn import(backup: &Value) -> Result<Value, ApiError> {
    api_post("/api/backup/import", backup).await
}
