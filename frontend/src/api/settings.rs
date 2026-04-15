use crate::api::client::{api_get, api_put, ApiError};
use crate::models::auth::*;

pub async fn get() -> Result<SettingsResponse, ApiError> {
    api_get("/api/settings").await
}

pub async fn update(req: &UpdateSettingsRequest) -> Result<SettingsResponse, ApiError> {
    api_put("/api/settings", req).await
}