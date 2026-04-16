use axum::{
    extract::State,
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
use crate::shared::OptionalQuery;

#[derive(serde::Deserialize)]
pub struct ReportQuery {
    pub token: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

pub async fn get_report(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<ReportQuery>,
    headers: HeaderMap,
) -> AppResult<Json<ReportResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let today = chrono::Utc::now().date_naive();
    let year = query.as_ref().and_then(|q| q.year).unwrap_or(today.year());
    let month = query.as_ref().and_then(|q| q.month).unwrap_or(today.month());

    let report = ReportService::get_full_report(&pool, claims.sub, year, month).await;

    Ok(Json(report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_query_deserialize_with_params() {
        let query = "year=2024&month=6&token=abc";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.year, Some(2024));
        assert_eq!(result.month, Some(6));
        assert_eq!(result.token.as_deref(), Some("abc"));
    }

    #[test]
    fn report_query_deserialize_empty() {
        let query = "";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.year.is_none());
        assert!(result.month.is_none());
    }

    #[test]
    fn report_query_defaults_to_current_date() {
        let today = chrono::Utc::now().date_naive();
        let year = None::<i32>.unwrap_or(today.year());
        let month = None::<u32>.unwrap_or(today.month());
        assert_eq!(year, today.year());
        assert_eq!(month, today.month());
    }
}