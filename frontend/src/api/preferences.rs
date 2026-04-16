use crate::api::client::{api_get, api_put, ApiError};
use crate::models::preferences::*;

pub async fn get() -> Result<PreferencesResponse, ApiError> {
    api_get("/api/preferences").await
}

pub async fn update(req: &UpdatePreferencesRequest) -> Result<PreferencesResponse, ApiError> {
    api_put("/api/preferences", req).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn preferences_api_paths() {
        assert!("/api/preferences".starts_with("/api/"));
    }

    #[test]
    fn update_preferences_request_serialization() {
        use crate::models::preferences::UpdatePreferencesRequest;
        let req = UpdatePreferencesRequest {
            theme: Some("dark".into()),
            subscriptions_view: None,
            first_day_of_week: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("dark"));
    }
}