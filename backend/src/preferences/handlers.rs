use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use sqlx::PgPool;

use crate::auth::service::AuthService;
use crate::preferences::models::{PreferencesResponse, UpdatePreferencesRequest};
use crate::preferences::service::PreferencesService;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;
use crate::shared::OptionalQuery;

#[derive(serde::Deserialize)]
pub struct PreferencesQuery {
    pub token: Option<String>,
}

pub async fn get_preferences(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<PreferencesQuery>,
    headers: HeaderMap,
) -> AppResult<Json<PreferencesResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;
    let preferences = PreferencesService::get_preferences(&pool, claims.sub).await?;
    Ok(Json(preferences))
}

pub async fn update_preferences(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<PreferencesQuery>,
    headers: HeaderMap,
    Json(req): Json<UpdatePreferencesRequest>,
) -> AppResult<Json<PreferencesResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;
    let preferences = PreferencesService::update_preferences(&pool, claims.sub, req).await?;
    Ok(Json(preferences))
}
#[cfg(test)]
mod tests {
    use super::*;

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
}
