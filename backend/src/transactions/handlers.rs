use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::shared::error::AppResult;
use crate::transactions::models::{CreateTransactionRequest, TransactionResponse, UpdateTransactionRequest};
use crate::transactions::service::TransactionService;

use crate::shared::{AuthQuery, OptionalQuery};
use crate::shared::auth_extractor::extract_token_from_request;

#[derive(serde::Deserialize)]
pub struct DateRangeQuery {
    pub token: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

pub async fn create(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateTransactionRequest>,
) -> AppResult<(StatusCode, Json<TransactionResponse>)> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let transaction = TransactionService::create(&pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(transaction.into())))
}

pub async fn list(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<TransactionResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let transactions = TransactionService::get_all(&pool, claims.sub).await?;

    Ok(Json(transactions.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<TransactionResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let transaction = TransactionService::get_by_id(&pool, claims.sub, id).await?;

    Ok(Json(transaction.into()))
}

pub async fn update(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateTransactionRequest>,
) -> AppResult<Json<TransactionResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let transaction = TransactionService::update(&pool, claims.sub, id, req).await?;

    Ok(Json(transaction.into()))
}

pub async fn delete(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    TransactionService::delete(&pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn by_date_range(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<DateRangeQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<TransactionResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let from_date = query.as_ref()
        .and_then(|q| q.from_date.as_ref())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let to_date = query.as_ref()
        .and_then(|q| q.to_date.as_ref())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let transactions = TransactionService::get_by_date_range(&pool, claims.sub, from_date, to_date).await?;

    Ok(Json(transactions.into_iter().map(Into::into).collect()))
}

/// Parse a date string in YYYY-MM-DD format, returning None on failure.
pub(crate) fn parse_date(date_str: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    // ── parse_date ──

    #[test]
    fn parse_date_valid() {
        let result = parse_date("2024-06-15");
        assert!(result.is_some());
        let d = result.unwrap();
        assert_eq!(d.year(), 2024);
        assert_eq!(d.month(), 6);
        assert_eq!(d.day(), 15);
    }

    #[test]
    fn parse_date_invalid_format() {
        assert!(parse_date("06/15/2024").is_none());
        assert!(parse_date("2024-13-01").is_none());
        assert!(parse_date("not-a-date").is_none());
        assert!(parse_date("").is_none());
    }

    #[test]
    fn parse_date_edge_cases() {
        assert!(parse_date("2024-02-29").is_some()); // leap year
        assert!(parse_date("2023-02-29").is_none()); // not a leap year
        assert!(parse_date("2024-12-31").is_some());
        assert!(parse_date("2024-01-01").is_some());
    }

    // ── DateRangeQuery deserialization ──

    #[test]
    fn date_range_query_deserialize() {
        let query = "from_date=2024-01-01&to_date=2024-12-31";
        let result: DateRangeQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.from_date.as_deref(), Some("2024-01-01"));
        assert_eq!(result.to_date.as_deref(), Some("2024-12-31"));
    }

    #[test]
    fn date_range_query_empty() {
        let query = "";
        let result: DateRangeQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.from_date.is_none());
        assert!(result.to_date.is_none());
    }

    #[test]
    fn date_range_query_partial() {
        let query = "from_date=2024-06-01";
        let result: DateRangeQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.from_date.as_deref(), Some("2024-06-01"));
        assert!(result.to_date.is_none());
    }
}