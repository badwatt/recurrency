use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use chrono::Datelike;
use sqlx::PgPool;

use crate::auth::service::AuthService;
use crate::reports::models::ReportResponse;
use crate::reports::service::ReportService;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;

#[derive(serde::Deserialize)]
pub struct ReportQuery {
    pub token: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

pub async fn get_report(
    State(pool): State<PgPool>,
    Query(query): Query<ReportQuery>,
    headers: HeaderMap,
) -> AppResult<Json<ReportResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.token.as_ref(), &headers)?).await?;

    let today = chrono::Utc::now().date_naive();
    let year = query.year.unwrap_or(today.year());
    let month = query.month.unwrap_or(today.month());

    let report = ReportService::get_full_report(&pool, claims.sub, year, month).await;

    Ok(Json(report))
}
