use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::Deserialize;
use serde_json::Value;

use crate::notifications::channels::{
    NotificationProvider, SummaryNotificationSettingsResponse, UpdateNotificationChannelRequest,
    UpdateSummaryNotificationSettingsRequest,
};
use crate::shared::app_state::AppState;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::{AppError, AppResult};
use crate::shared::{AuthQuery, OptionalQuery};

async fn authenticated_user(
    state: &AppState,
    query: Option<AuthQuery>,
    headers: &HeaderMap,
) -> AppResult<uuid::Uuid> {
    let token = extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), headers)?;
    Ok(state.auth_service.verify_token(&token).await?.sub)
}

fn provider(value: &str) -> AppResult<NotificationProvider> {
    NotificationProvider::parse(value)
        .ok_or_else(|| AppError::BadRequest("Unknown notification provider".to_string()))
}

pub async fn list_channels(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<crate::notifications::channels::NotificationChannelResponse>>> {
    let user_id = authenticated_user(&state, query, &headers).await?;
    Ok(Json(
        state
            .notification_settings_service
            .list_channels(user_id)
            .await?,
    ))
}

pub async fn update_channel(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(provider_name): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateNotificationChannelRequest>,
) -> AppResult<Json<crate::notifications::channels::NotificationChannelResponse>> {
    let user_id = authenticated_user(&state, query, &headers).await?;
    let provider = provider(&provider_name)?;
    Ok(Json(
        state
            .notification_settings_service
            .update_channel(user_id, provider, req)
            .await?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct TestChannelRequest {
    #[serde(default)]
    pub config: Option<Value>,
}

pub async fn test_channel(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(provider_name): Path<String>,
    headers: HeaderMap,
    Json(req): Json<TestChannelRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = authenticated_user(&state, query, &headers).await?;
    let provider = provider(&provider_name)?;
    state
        .notification_settings_service
        .test_channel(user_id, provider, req.config)
        .await?;
    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn get_summary(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<SummaryNotificationSettingsResponse>> {
    let user_id = authenticated_user(&state, query, &headers).await?;
    Ok(Json(
        state
            .notification_settings_service
            .get_summary(user_id)
            .await?,
    ))
}

pub async fn update_summary(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<UpdateSummaryNotificationSettingsRequest>,
) -> AppResult<Json<SummaryNotificationSettingsResponse>> {
    let user_id = authenticated_user(&state, query, &headers).await?;
    Ok(Json(
        state
            .notification_settings_service
            .update_summary(user_id, req)
            .await?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_parser_rejects_unknown() {
        assert!(provider("ntfy").is_ok());
        assert!(provider("unknown").is_err());
    }

    #[test]
    fn test_request_config_is_optional() {
        let request: TestChannelRequest = serde_json::from_str("{}").unwrap();
        assert!(request.config.is_none());
    }
}
