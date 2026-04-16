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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_query_deserialize() {
        let query = "year=2024&month=6&token=abc";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.year, Some(2024));
        assert_eq!(result.month, Some(6));
        assert_eq!(result.token.as_deref(), Some("abc"));
    }

    #[test]
    fn calendar_query_empty() {
        let query = "";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.year.is_none());
        assert!(result.month.is_none());
        assert!(result.token.is_none());
    }

    #[test]
    fn calendar_query_partial_year_only() {
        let query = "year=2024";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.year, Some(2024));
        assert!(result.month.is_none());
    }

    #[test]
    fn calendar_defaults_to_current_date() {
        let today = chrono::Utc::now().date_naive();
        let year = None::<i32>.unwrap_or(today.year());
        let month = None::<u32>.unwrap_or(today.month());
        assert_eq!(year, today.year());
        assert_eq!(month, today.month());
    }

    #[test]
    fn calendar_item_serialization() {
        let item = CalendarItem {
            date: chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            subscription_id: Uuid::new_v4(),
            name: "Netflix".to_string(),
            amount: rust_decimal::Decimal::from(1599),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            logo_url: None,
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["name"], "Netflix");
        assert!(json.get("date").is_some());
    }
}