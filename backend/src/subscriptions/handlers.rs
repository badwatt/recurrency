use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Datelike;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::auth::service::AuthService;
use crate::shared::error::AppResult;
use crate::subscriptions::models::{
    CreateSubscriptionRequest, SubscriptionResponse, UpdateSubscriptionRequest,
};
use crate::subscriptions::service::{SubscriptionService, CalendarItem};

use crate::shared::{AuthQuery, OptionalQuery};
use crate::shared::auth_extractor::extract_token_from_request;

#[derive(serde::Deserialize)]
pub struct CalendarQuery {
    pub token: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

pub async fn create(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateSubscriptionRequest>,
) -> AppResult<(StatusCode, Json<SubscriptionResponse>)> {
    req.validate()?;

    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let subscription = SubscriptionService::create(&pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(subscription.into())))
}

pub async fn list(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<SubscriptionResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let subscriptions = SubscriptionService::get_all(&pool, claims.sub).await?;

    Ok(Json(subscriptions.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<SubscriptionResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let subscription = SubscriptionService::get_by_id(&pool, claims.sub, id).await?;

    Ok(Json(subscription.into()))
}

pub async fn update(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateSubscriptionRequest>,
) -> AppResult<Json<SubscriptionResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let subscription = SubscriptionService::update(&pool, claims.sub, id, req).await?;

    Ok(Json(subscription.into()))
}

pub async fn delete(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    SubscriptionService::delete(&pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn upcoming(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<SubscriptionResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let subscriptions = SubscriptionService::get_upcoming(&pool, claims.sub, 7).await?;

    Ok(Json(subscriptions.into_iter().map(Into::into).collect()))
}

pub async fn calendar(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<CalendarQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<CalendarItem>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let today = chrono::Utc::now().date_naive();
    let year = query.as_ref().and_then(|q| q.year).unwrap_or(today.year());
    let month = query.as_ref().and_then(|q| q.month).unwrap_or(today.month());

    let items = SubscriptionService::get_calendar(&pool, claims.sub, year, month).await?;

    Ok(Json(items))
}