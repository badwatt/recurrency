use crate::api::client::{api_get, api_put, ApiError};
use crate::models::auth::*;

pub async fn get() -> Result<SettingsResponse, ApiError> {
    api_get("/api/settings").await
}

pub async fn update(req: &UpdateSettingsRequest) -> Result<SettingsResponse, ApiError> {
    api_put("/api/settings", req).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn settings_api_paths() {
        assert!("/api/settings".starts_with("/api/"));
    }

    #[test]
    fn update_settings_request_serialization() {
        use crate::models::auth::UpdateSettingsRequest;
        let req = UpdateSettingsRequest {
            name: Some("Alice".into()),
            currency: Some("USD".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Alice"));
        assert!(json.contains("USD"));
    }
}