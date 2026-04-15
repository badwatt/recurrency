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