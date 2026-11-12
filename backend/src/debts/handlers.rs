use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::debts::models::{
    CreateDebtRequest, DebtPayment, DebtResponse, RecordDebtPaymentRequest, UpdateDebtRequest,
};
use crate::shared::app_state::AppState;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;
use crate::shared::{AuthQuery, OptionalQuery};

async fn user_id(
    state: &AppState,
    query: Option<AuthQuery>,
    headers: &HeaderMap,
) -> AppResult<Uuid> {
    let token = extract_token_from_request(query.and_then(|q| q.token).as_ref(), headers)?;
    Ok(state.auth_service.verify_token(&token).await?.sub)
}

pub async fn create(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateDebtRequest>,
) -> AppResult<(StatusCode, Json<DebtResponse>)> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok((
        StatusCode::CREATED,
        Json(state.debt_service.create(user_id, req).await?),
    ))
}

pub async fn list(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<DebtResponse>>> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok(Json(state.debt_service.get_all(user_id).await?))
}

pub async fn get(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<DebtResponse>> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok(Json(state.debt_service.get_by_id(user_id, id).await?))
}

pub async fn update(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateDebtRequest>,
) -> AppResult<Json<DebtResponse>> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok(Json(state.debt_service.update(user_id, id, req).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let user_id = user_id(&state, query, &headers).await?;
    state.debt_service.delete(user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn record_payment(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<RecordDebtPaymentRequest>,
) -> AppResult<Json<DebtResponse>> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok(Json(
        state.debt_service.record_payment(user_id, id, req).await?,
    ))
}

pub async fn payments(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<DebtPayment>>> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok(Json(state.debt_service.payments(user_id, id).await?))
}
