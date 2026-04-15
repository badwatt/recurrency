use crate::api::client::{api_get, api_put, ApiError};
use crate::models::preferences::*;

pub async fn get() -> Result<PreferencesResponse, ApiError> {
    api_get("/api/preferences").await
}

pub async fn update(req: &UpdatePreferencesRequest) -> Result<PreferencesResponse, ApiError> {
    api_put("/api/preferences", req).await
}