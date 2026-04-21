use axum::{extract::State, http::HeaderMap, Json};

use crate::auth::service::AuthService;
use crate::preferences::models::{PreferencesResponse, UpdatePreferencesRequest};
use crate::preferences::service::PreferencesService;
use crate::shared::app_state::AppState;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;
use crate::shared::OptionalQuery;

#[derive(serde::Deserialize)]
pub struct PreferencesQuery {
    pub token: Option<String>,
}

pub async fn get_preferences(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<PreferencesQuery>,
    headers: HeaderMap,
) -> AppResult<Json<PreferencesResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;
    let preferences = PreferencesService::get_preferences(&state.pool, claims.sub).await?;
    Ok(Json(preferences))
}

pub async fn update_preferences(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<PreferencesQuery>,
    headers: HeaderMap,
    Json(req): Json<UpdatePreferencesRequest>,
) -> AppResult<Json<PreferencesResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;
    let preferences = PreferencesService::update_preferences(&state.pool, claims.sub, req).await?;
    Ok(Json(preferences))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::models::{PreferencesResponse, UpdatePreferencesRequest};

    #[test]
    fn preferences_query_deserialize() {
        let query = "token=my-token";
        let result: PreferencesQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("my-token"));
    }

    #[test]
    fn preferences_query_empty() {
        let query = "";
        let result: PreferencesQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_none());
    }

    #[test]
    fn preferences_query_empty_token() {
        let query = "token=";
        let result: PreferencesQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_some());
        assert!(result.token.as_ref().unwrap().is_empty());
    }

    #[test]
    fn update_preferences_request_deserialize() {
        let json = r#"{"theme":"dark","subscriptions_view":"list"}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.theme.as_deref(), Some("dark"));
        assert_eq!(req.subscriptions_view.as_deref(), Some("list"));
    }

    #[test]
    fn update_preferences_request_partial() {
        let json = r#"{"theme":"light"}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.theme.as_deref(), Some("light"));
        assert!(req.subscriptions_view.is_none());
    }

    #[test]
    fn update_preferences_request_empty() {
        let json = r#"{}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert!(req.theme.is_none());
        assert!(req.subscriptions_view.is_none());
    }

    #[test]
    fn update_preferences_request_view_only() {
        let json = r#"{"subscriptions_view":"compact"}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert!(req.theme.is_none());
        assert_eq!(req.subscriptions_view.as_deref(), Some("compact"));
    }

    #[test]
    fn update_preferences_all_theme_values() {
        for theme in &["system", "light", "dark", "midnight"] {
            let json = format!(r#"{{"theme":"{}"}}"#, theme);
            let req: UpdatePreferencesRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(req.theme.as_deref(), Some(*theme));
        }
    }

    #[test]
    fn update_preferences_all_view_values() {
        for view in &["grid", "list", "compact", "table"] {
            let json = format!(r#"{{"subscriptions_view":"{}"}}"#, view);
            let req: UpdatePreferencesRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(req.subscriptions_view.as_deref(), Some(*view));
        }
    }

    #[test]
    fn preferences_response_serialization() {
        let resp = PreferencesResponse {
            theme: "dark".to_string(),
            subscriptions_view: "list".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("dark"));
        assert!(json.contains("list"));
    }

    #[test]
    fn preferences_response_default_values() {
        let resp = PreferencesResponse::default();
        assert_eq!(resp.theme, "system");
        assert_eq!(resp.subscriptions_view, "grid");
    }

    #[test]
    fn preferences_response_clone() {
        let resp = PreferencesResponse::default();
        let cloned = resp.clone();
        assert_eq!(cloned.theme, resp.theme);
        assert_eq!(cloned.subscriptions_view, resp.subscriptions_view);
    }

    #[test]
    fn update_preferences_empty_strings() {
        let json = r#"{"theme":"","subscriptions_view":""}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert!(req.theme.is_some()); // empty string is still a value
        assert!(req.subscriptions_view.is_some());
    }

    #[test]
    fn update_preferences_unicode_theme() {
        let json = r#"{"theme":"テーマ"}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.theme.as_deref(), Some("テーマ"));
    }

    #[test]
    fn update_preferences_unicode_view() {
        let json = r#"{"subscriptions_view":"视图"}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.subscriptions_view.as_deref(), Some("视图"));
    }
}
