use crate::api::client::{api_get, api_put, ApiError};
use crate::models::preferences::*;

pub async fn get() -> Result<PreferencesResponse, ApiError> {
    api_get("/api/preferences").await
}

pub async fn update(req: &UpdatePreferencesRequest) -> Result<PreferencesResponse, ApiError> {
    log::info!("Updating preferences: {:?}", req);
    api_put("/api/preferences", req).await
}

#[cfg(test)]
mod tests {
    use crate::models::preferences::*;

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
            date_format: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("dark"));
    }

    #[test]
    fn preferences_response_default() {
        let prefs = PreferencesResponse::default();
        assert_eq!(prefs.theme, "system");
        assert_eq!(prefs.subscriptions_view, "grid");
        assert_eq!(prefs.first_day_of_week, "monday");
    }

    #[test]
    fn preferences_response_serialization() {
        let prefs = PreferencesResponse {
            theme: "light".into(),
            subscriptions_view: "list".into(),
            first_day_of_week: "sunday".into(),
            date_format: "DD-MM-YYYY".into(),
        };
        let json = serde_json::to_string(&prefs).unwrap();
        assert!(json.contains("light"));
        assert!(json.contains("list"));
        assert!(json.contains("sunday"));
    }

    #[test]
    fn update_request_view_only() {
        let req = UpdatePreferencesRequest {
            theme: None,
            subscriptions_view: Some("table".into()),
            first_day_of_week: None,
            date_format: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("table"));
    }

    #[test]
    fn update_request_first_day_of_week() {
        let req = UpdatePreferencesRequest {
            theme: None,
            subscriptions_view: None,
            first_day_of_week: Some("sunday".into()),
            date_format: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("sunday"));
    }

    #[test]
    fn update_request_all_fields() {
        let req = UpdatePreferencesRequest {
            theme: Some("dark".into()),
            subscriptions_view: Some("grid".into()),
            first_day_of_week: Some("monday".into()),
            date_format: Some("DD-MM-YYYY".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("dark"));
        assert!(json.contains("grid"));
        assert!(json.contains("monday"));
    }
}
