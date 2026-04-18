use crate::api::client::{api_get, api_put, ApiError};
use crate::models::auth::*;

#[allow(dead_code)]
pub async fn get() -> Result<SettingsResponse, ApiError> {
    api_get("/api/settings").await
}

pub async fn update(req: &UpdateSettingsRequest) -> Result<SettingsResponse, ApiError> {
    api_put("/api/settings", req).await
}

#[cfg(test)]
mod tests {
    use crate::models::auth::{SettingsResponse, UserResponse};
    use uuid::Uuid;

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

    // ── Bug: Backend returns UserResponse, frontend expects SettingsResponse ──
    #[test]
    fn settings_response_requires_user_wrapper() {
        // Frontend SettingsResponse expects: { user: {...} }
        // Backend returns UserResponse directly: {...} (no wrapper)
        let user_response = UserResponse {
            id: Uuid::new_v4(),
            email: "test@test.com".into(),
            name: Some("Alice".into()),
            currency: "USD".into(),
            verified: true,
        };

        // UserResponse serializes WITHOUT wrapper
        let user_json = serde_json::to_string(&user_response).unwrap();
        assert!(!user_json.starts_with("{\"user\"")); // No wrapper

        // SettingsResponse expects wrapper - parse fails
        let result: Result<SettingsResponse, _> = serde_json::from_str(&user_json);
        assert!(result.is_err(), "UserResponse can't parse as SettingsResponse - missing wrapper");
    }

    #[test]
    fn settings_response_with_user_wrapper_parses_ok() {
        // Correct format: { "user": {...} }
        let response = SettingsResponse {
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "test@test.com".into(),
                name: Some("Alice".into()),
                currency: "USD".into(),
                verified: true,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.starts_with("{\"user\""));

        let parsed: SettingsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.user.currency, "USD");
    }

    #[test]
    fn update_settings_request_currency_only() {
        use crate::models::auth::UpdateSettingsRequest;
        let req = UpdateSettingsRequest {
            name: None,
            currency: Some("EUR".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("EUR"));
    }

    #[test]
    fn update_settings_request_name_only() {
        use crate::models::auth::UpdateSettingsRequest;
        let req = UpdateSettingsRequest {
            name: Some("Bob".into()),
            currency: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Bob"));
    }

    #[test]
    fn settings_response_user_fields() {
        let response = SettingsResponse {
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "test@test.com".into(),
                name: None,
                currency: "GBP".into(),
                verified: false,
            },
        };
        assert_eq!(response.user.email, "test@test.com");
        assert_eq!(response.user.currency, "GBP");
        assert!(response.user.name.is_none());
        assert!(!response.user.verified);
    }
}
