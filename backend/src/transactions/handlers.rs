use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::shared::error::AppResult;
use crate::transactions::models::{CreateTransactionRequest, TransactionResponse, UpdateTransactionRequest};
use crate::transactions::service::TransactionService;

#[derive(serde::Deserialize)]
pub struct AuthQuery {
    pub token: String,
}

#[derive(serde::Deserialize)]
pub struct DateRangeQuery {
    pub token: String,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

pub async fn create(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Json(req): Json<CreateTransactionRequest>,
) -> AppResult<(StatusCode, Json<TransactionResponse>)> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let transaction = TransactionService::create(&pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(transaction.into())))
}

pub async fn list(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
) -> AppResult<Json<Vec<TransactionResponse>>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let transactions = TransactionService::get_all(&pool, claims.sub).await?;

    Ok(Json(transactions.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<TransactionResponse>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let transaction = TransactionService::get_by_id(&pool, claims.sub, id).await?;

    Ok(Json(transaction.into()))
}

pub async fn update(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTransactionRequest>,
) -> AppResult<Json<TransactionResponse>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let transaction = TransactionService::update(&pool, claims.sub, id, req).await?;

    Ok(Json(transaction.into()))
}

pub async fn delete(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let claims = AuthService::verify_token(&auth.token).await?;

    TransactionService::delete(&pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn by_date_range(
    State(pool): State<PgPool>,
    Query(query): Query<DateRangeQuery>,
) -> AppResult<Json<Vec<TransactionResponse>>> {
    let claims = AuthService::verify_token(&query.token).await?;

    let from_date = query.from_date
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let to_date = query.to_date
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let transactions = TransactionService::get_by_date_range(&pool, claims.sub, from_date, to_date).await?;

    Ok(Json(transactions.into_iter().map(Into::into).collect()))
}
