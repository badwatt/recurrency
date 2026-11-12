use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::currency::service::{RatesResponse, get_rates};
use crate::shared::app_state::AppState;
use crate::shared::error::AppResult;

#[derive(Debug, Deserialize)]
pub struct RatesQuery {
    pub base: Option<String>,
}

pub async fn rates(
    State(_state): State<AppState>,
    Query(query): Query<RatesQuery>,
) -> AppResult<Json<RatesResponse>> {
    Ok(Json(get_rates(query.base.as_deref().unwrap_or("EUR")).await?))
}
