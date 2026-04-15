use crate::api::client::{api_get, api_post, api_put, api_delete, ApiError};
use crate::models::subscription::*;

pub async fn list() -> Result<Vec<Subscription>, ApiError> {
    api_get("/api/subscriptions").await
}

pub async fn get(id: &uuid::Uuid) -> Result<Subscription, ApiError> {
    api_get(&format!("/api/subscriptions/{}", id)).await
}

pub async fn create(req: &CreateSubscriptionRequest) -> Result<Subscription, ApiError> {
    api_post("/api/subscriptions", req).await
}

pub async fn update(id: &uuid::Uuid, req: &UpdateSubscriptionRequest) -> Result<Subscription, ApiError> {
    api_put(&format!("/api/subscriptions/{}", id), req).await
}

pub async fn delete(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/subscriptions/{}", id)).await
}

pub async fn upcoming() -> Result<Vec<Subscription>, ApiError> {
    api_get("/api/subscriptions/upcoming").await
}

pub async fn calendar_month(year: i32, month: u32) -> Result<Vec<CalendarItem>, ApiError> {
    api_get(&format!("/api/subscriptions/calendar?year={}&month={}", year, month)).await
}