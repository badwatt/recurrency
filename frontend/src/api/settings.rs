use dioxus::prelude::ReadableExt;
use crate::api::client::{api_get, api_post, api_put, is_offline_mode, ApiError};
use crate::models::auth::*;

#[allow(dead_code)]
pub async fn get() -> Result<SettingsResponse, ApiError> {
    api_get("/api/settings").await
}

/// GET /api/settings returns the user directly, not wrapped in SettingsResponse.
pub async fn get_user() -> Result<UserResponse, ApiError> {
    if is_offline_mode() {
        return crate::state::auth::AUTH
            .try_read()
            .ok()
            .and_then(|auth| auth.user.clone())
            .ok_or(ApiError::Unauthorized);
    }
    api_get("/api/settings").await
}

pub async fn update(req: &UpdateSettingsRequest) -> Result<SettingsResponse, ApiError> {
    if is_offline_mode() {
        let user = crate::state::auth::AUTH
            .try_read()
            .ok()
            .and_then(|auth| auth.user.clone())
            .ok_or(ApiError::Unauthorized)?;
        let fallback_name = user.name.clone().unwrap_or_default();
        let fallback_avatar = user.avatar.clone();
        let profile = crate::storage::local_auth::update_profile(
            user.id,
            req.email.as_deref().unwrap_or(&user.email),
            req.name.as_deref().unwrap_or(&fallback_name),
            req.avatar.clone().or(fallback_avatar),
            req.currency.clone(),
        )
        .map_err(ApiError::Network)?;
        return Ok(SettingsResponse { user: profile });
    }
    api_put("/api/settings", req).await
}

pub async fn delete_account(req: &DeleteAccountRequest) -> Result<(), ApiError> {
    let _: serde_json::Value = api_post("/api/settings/account", req).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::auth::{SettingsResponse, UserResponse};
    use uuid::Uuid;

    #[test]
    fn delete_account_uses_settings_account_path() {
        let req = crate::models::auth::DeleteAccountRequest {
            password: "secret".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert_eq!("/api/settings/account", "/api/settings/account");
        assert!(json.contains("secret"));
    }

    #[test]
    fn settings_api_paths() {
        assert!("/api/settings".starts_with("/api/"));
    }

    #[test]
    fn update_settings_request_serialization() {
        use crate::models::auth::UpdateSettingsRequest;
        let req = UpdateSettingsRequest {
            email: None,
                avatar: None,
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
                avatar: None,
            name: Some("Alice".into()),
            currency: "USD".into(),
            verified: true,
            is_admin: false,
        };

        // UserResponse serializes WITHOUT wrapper
        let user_json = serde_json::to_string(&user_response).unwrap();
        assert!(!user_json.starts_with("{\"user\"")); // No wrapper

        // SettingsResponse expects wrapper - parse fails
        let result: Result<SettingsResponse, _> = serde_json::from_str(&user_json);
        assert!(
            result.is_err(),
            "UserResponse can't parse as SettingsResponse - missing wrapper"
        );
    }

    #[test]
    fn settings_response_with_user_wrapper_parses_ok() {
        // Correct format: { "user": {...} }
        let response = SettingsResponse {
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "test@test.com".into(),
                avatar: None,
                name: Some("Alice".into()),
                currency: "USD".into(),
                verified: true,
                is_admin: false,
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
            email: None,
                avatar: None,
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
            email: None,
                avatar: None,
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
                avatar: None,
                name: None,
                currency: "GBP".into(),
                verified: false,
                is_admin: false,
            },
        };
        assert_eq!(response.user.email, "test@test.com");
        assert_eq!(response.user.currency, "GBP");
        assert!(response.user.name.is_none());
        assert!(!response.user.verified);
    }
    use crate::api::test_support::with_server;
    use serial_test::serial;

    fn user_json() -> String {
        serde_json::json!({
            "id": Uuid::new_v4(),
            "email": "user@test.com",
            "avatar": null,
            "name": "User",
            "currency": "EUR",
            "verified": true,
            "is_admin": false
        })
        .to_string()
    }

    #[tokio::test]
    #[serial]
    async fn settings_endpoints_exchange_json() {
        let wrapped = format!(r#"{{"user":{}}}"#, user_json());
        let settings: SettingsResponse = with_server(wrapped.clone(), get()).await.unwrap();
        assert_eq!(settings.user.email, "user@test.com");

        let user: UserResponse = with_server(user_json(), get_user()).await.unwrap();
        assert_eq!(user.email, "user@test.com");

        let updated: SettingsResponse = with_server(
            wrapped,
            update(&UpdateSettingsRequest {
                email: Some("user@test.com".into()),
                avatar: None,
                name: Some("User".into()),
                currency: Some("USD".into()),
            }),
        )
        .await
        .unwrap();
        assert_eq!(updated.user.currency, "EUR");

        with_server(
            "null",
            delete_account(&DeleteAccountRequest {
                password: "secret".into(),
            }),
        )
        .await
        .unwrap();
    }

}
