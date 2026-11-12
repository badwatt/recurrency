use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::currency::service::{
    CurrencySettingsResponse, RatesResponse, UpdateCurrencySettingsRequest, get_currency_settings,
    get_rates, get_rates_for_user, update_currency_settings,
};
use crate::shared::app_state::AppState;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;

#[derive(Debug, Deserialize)]
pub struct RatesQuery {
    pub base: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SettingsQuery {
    pub token: Option<String>,
}

async fn optional_user_id(
    state: &AppState,
    query_token: Option<&String>,
    headers: &HeaderMap,
) -> Option<Uuid> {
    let token = extract_token_from_request(query_token, headers).ok()?;
    state.auth_service.verify_token(&token).await.ok().map(|claims| claims.sub)
}

pub async fn rates(
    State(state): State<AppState>,
    Query(query): Query<RatesQuery>,
    headers: HeaderMap,
) -> AppResult<Json<RatesResponse>> {
    let base = query.base.as_deref().unwrap_or("EUR");
    let response = match optional_user_id(&state, query.token.as_ref(), &headers).await {
        Some(user_id) => get_rates_for_user(&state.pool, user_id, base).await?,
        None => get_rates(base).await?,
    };
    Ok(Json(response))
}

pub async fn get_settings(
    State(state): State<AppState>,
    Query(query): Query<SettingsQuery>,
    headers: HeaderMap,
) -> AppResult<Json<CurrencySettingsResponse>> {
    let token = extract_token_from_request(query.token.as_ref(), &headers)?;
    let claims = state.auth_service.verify_token(&token).await?;
    Ok(Json(
        get_currency_settings(&state.pool, claims.sub).await?,
    ))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Query(query): Query<SettingsQuery>,
    headers: HeaderMap,
    Json(req): Json<UpdateCurrencySettingsRequest>,
) -> AppResult<Json<CurrencySettingsResponse>> {
    let token = extract_token_from_request(query.token.as_ref(), &headers)?;
    let claims = state.auth_service.verify_token(&token).await?;
    Ok(Json(
        update_currency_settings(&state.pool, claims.sub, req).await?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rates_query_deserialize() {
        let query: RatesQuery = serde_urlencoded::from_str("base=usd&token=abc").unwrap();
        assert_eq!(query.base.as_deref(), Some("usd"));
        assert_eq!(query.token.as_deref(), Some("abc"));
    }

    #[test]
    fn settings_query_deserialize() {
        let query: SettingsQuery = serde_urlencoded::from_str("token=abc").unwrap();
        assert_eq!(query.token.as_deref(), Some("abc"));
    }
}
