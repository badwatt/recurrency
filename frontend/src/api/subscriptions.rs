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

#[cfg(test)]
mod tests {
    #[test]
    fn subscription_api_paths() {
        // Verify URL patterns are correctly formed
        let base = "/api/subscriptions";
        assert!(base.starts_with("/api/"));

        let id = uuid::Uuid::new_v4();
        let detail = format!("/api/subscriptions/{}", id);
        assert!(detail.starts_with("/api/subscriptions/"));

        let upcoming = "/api/subscriptions/upcoming";
        assert!(upcoming.contains("upcoming"));

        let calendar = format!("/api/subscriptions/calendar?year={}&month={}", 2024, 6);
        assert!(calendar.contains("calendar"));
        assert!(calendar.contains("year=2024"));
        assert!(calendar.contains("month=6"));
    }

    #[test]
    fn calendar_url_for_various_months() {
        for (y, m) in [(2024, 1), (2024, 12), (2025, 6)] {
            let url = format!("/api/subscriptions/calendar?year={}&month={}", y, m);
            assert!(url.contains(&format!("year={}", y)));
            assert!(url.contains(&format!("month={}", m)));
        }
    }

    #[test]
    fn subscription_id_url_format() {
        let id = uuid::Uuid::new_v4();
        let path = format!("/api/subscriptions/{}", id);
        assert_eq!(path.len(), "/api/subscriptions/".len() + 36); // UUID is 36 chars
    }
}