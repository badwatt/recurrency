use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::auth::service::AuthService;
use crate::shared::error::AppResult;
use crate::subscriptions::models::{
    CreateSubscriptionRequest, SubscriptionResponse, UpdateSubscriptionRequest,
};
use crate::subscriptions::service::{SubscriptionService, CalendarItem};

#[derive(serde::Deserialize)]
pub struct AuthQuery {
    pub token: String,
}

#[derive(serde::Deserialize)]
pub struct CalendarQuery {
    pub token: String,
    pub year: i32,
    pub month: u32,
}

pub async fn create(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> AppResult<(StatusCode, Json<SubscriptionResponse>)> {
    req.validate()?;

    let claims = AuthService::verify_token(&auth.token).await?;

    let subscription = SubscriptionService::create(&pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(subscription.into())))
}

pub async fn list(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
) -> AppResult<Json<Vec<SubscriptionResponse>>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let subscriptions = SubscriptionService::get_all(&pool, claims.sub).await?;

    Ok(Json(subscriptions.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<SubscriptionResponse>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let subscription = SubscriptionService::get_by_id(&pool, claims.sub, id).await?;

    Ok(Json(subscription.into()))
}

pub async fn update(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSubscriptionRequest>,
) -> AppResult<Json<SubscriptionResponse>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let subscription = SubscriptionService::update(&pool, claims.sub, id, req).await?;

    Ok(Json(subscription.into()))
}

pub async fn delete(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let claims = AuthService::verify_token(&auth.token).await?;

    SubscriptionService::delete(&pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn upcoming(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
) -> AppResult<Json<Vec<SubscriptionResponse>>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let subscriptions = SubscriptionService::get_upcoming(&pool, claims.sub, 7).await?;

    Ok(Json(subscriptions.into_iter().map(Into::into).collect()))
}

pub async fn calendar(
    State(pool): State<PgPool>,
    Query(query): Query<CalendarQuery>,
) -> AppResult<Json<Vec<CalendarItem>>> {
    let claims = AuthService::verify_token(&query.token).await?;

    let items = SubscriptionService::get_calendar(&pool, claims.sub, query.year, query.month).await?;

    Ok(Json(items))
}
