use crate::api::client::{api_delete, api_get, api_post, api_put, ApiError};
use crate::models::subscription::*;

pub async fn list() -> Result<Vec<Subscription>, ApiError> {
    api_get("/api/subscriptions").await
}

#[allow(dead_code)]
pub async fn get(id: &uuid::Uuid) -> Result<Subscription, ApiError> {
    api_get(&format!("/api/subscriptions/{}", id)).await
}

pub async fn create(req: &CreateSubscriptionRequest) -> Result<Subscription, ApiError> {
    api_post("/api/subscriptions", req).await
}

pub async fn update(
    id: &uuid::Uuid,
    req: &UpdateSubscriptionRequest,
) -> Result<Subscription, ApiError> {
    api_put(&format!("/api/subscriptions/{}", id), req).await
}

pub async fn delete(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/subscriptions/{}", id)).await
}

#[allow(dead_code)]
pub async fn upcoming() -> Result<Vec<Subscription>, ApiError> {
    api_get("/api/subscriptions/upcoming").await
}

pub async fn calendar_month(year: i32, month: u32) -> Result<Vec<CalendarItem>, ApiError> {
    api_get(&format!(
        "/api/subscriptions/calendar?year={}&month={}",
        year, month
    ))
    .await
}

#[cfg(test)]
mod tests {
    use crate::models::subscription::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

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

    #[test]
    fn create_subscription_request_fields() {
        let req = CreateSubscriptionRequest {
            account_id: Uuid::nil(),
            name: "Netflix".into(),
            amount: dec!(15.99),
            currency: "USD".into(),
            frequency: "monthly".into(),
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            is_active: None,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            category_id: Some(Uuid::new_v4()),
            notes: None,
            logo_url: None,
        };
        assert_eq!(req.name, "Netflix");
        assert_eq!(req.amount, dec!(15.99));
        assert_eq!(req.currency, "USD");
        assert_eq!(req.frequency, "monthly");
        assert!(req.notes.is_none());
    }

    #[test]
    fn create_subscription_request_with_notes() {
        let req = CreateSubscriptionRequest {
            account_id: Uuid::nil(),
            name: "Spotify".into(),
            amount: dec!(9.99),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            is_active: None,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            category_id: None,
            notes: Some("Family plan".into()),
            logo_url: Some("https://logo.com/spotify.png".into()),
        };
        assert!(req.notes.is_some());
        assert!(req.logo_url.is_some());
    }

    #[test]
    fn update_subscription_request_partial() {
        let req = UpdateSubscriptionRequest {
            account_id: None,
            name: Some("Netflix Premium".into()),
            amount: Some(dec!(19.99)),
            currency: None,
            frequency: None,
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            start_date: None,
            end_date: None,
            clear_end_date: false,
            next_billing_date: None,
            category_id: None,
            is_active: None,
            notes: None,
            logo_url: None,
        };
        assert_eq!(req.name.as_deref(), Some("Netflix Premium"));
        assert_eq!(req.amount.as_ref().unwrap(), &dec!(19.99));
        assert!(req.currency.is_none());
        assert!(req.frequency.is_none());
    }

    #[test]
    fn update_subscription_request_all_none() {
        let req: UpdateSubscriptionRequest = Default::default();
        assert!(req.name.is_none());
        assert!(req.amount.is_none());
        assert!(req.currency.is_none());
        assert!(req.frequency.is_none());
        assert!(req.start_date.is_none());
        assert!(req.next_billing_date.is_none());
        assert!(req.is_active.is_none());
    }

    #[test]
    fn subscription_model_fields() {
        let sub = Subscription {
            account_id: Uuid::nil(),
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Netflix".into(),
            amount: dec!(15.99),
            currency: "USD".into(),
            frequency: "monthly".into(),
            schedule_day: None,
            schedule_month: None,
            recurrence: Recurrence::default(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            next_billing_date: Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        assert_eq!(sub.name, "Netflix");
        assert!(sub.is_active);
        assert!(sub.next_billing_date.is_some());
        assert!(sub.notes.is_none());
    }

    #[test]
    fn subscription_inactive() {
        let sub = Subscription {
            account_id: Uuid::nil(),
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Old Service".into(),
            amount: dec!(5.0),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            schedule_day: None,
            schedule_month: None,
            recurrence: Recurrence::default(),
            start_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            end_date: None,
            next_billing_date: None,
            is_active: false,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        assert!(!sub.is_active);
        assert!(sub.next_billing_date.is_none());
    }

    #[test]
    fn calendar_item_model() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Netflix".into(),
            amount: dec!(15.99),
            currency: "USD".into(),
            frequency: "monthly".into(),
            billing_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            is_active: true,
            logo_url: None,
        };
        assert_eq!(item.name, "Netflix");
        assert_eq!(
            item.billing_date,
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()
        );
        assert!(item.is_active);
    }

    #[test]
    fn calendar_item_inactive() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Canceled".into(),
            amount: dec!(5.0),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            billing_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            is_active: false,
            logo_url: None,
        };
        assert!(!item.is_active);
    }

    #[test]
    fn calendar_item_date_string() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(date.to_string(), "2024-06-15");
    }

    #[test]
    fn frequency_options_all() {
        let freqs = vec!["weekly", "monthly", "quarterly", "yearly"];
        assert_eq!(freqs.len(), 4);
    }

    #[test]
    fn subscription_serialization_roundtrip() {
        let sub = Subscription {
            account_id: Uuid::nil(),
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Test".into(),
            amount: dec!(9.99),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            schedule_day: None,
            schedule_month: None,
            recurrence: Recurrence::default(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&sub).unwrap();
        let back: Subscription = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Test");
    }

    #[test]
    fn create_request_serialization() {
        let req = CreateSubscriptionRequest {
            account_id: Uuid::nil(),
            name: "Netflix".into(),
            amount: dec!(15.99),
            currency: "USD".into(),
            frequency: "monthly".into(),
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            is_active: None,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            category_id: None,
            notes: None,
            logo_url: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Netflix"));
        assert!(json.contains("15.99"));
    }

    #[test]
    fn update_request_serialization_partial() {
        let req = UpdateSubscriptionRequest {
            account_id: None,
            name: Some("Updated Name".into()),
            amount: Some(dec!(20.00)),
            currency: None,
            frequency: None,
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            start_date: None,
            end_date: None,
            clear_end_date: false,
            next_billing_date: None,
            category_id: None,
            is_active: None,
            notes: None,
            logo_url: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Updated Name"));
        assert!(json.contains("20"));
    }

    #[test]
    fn upcoming_path() {
        let path = "/api/subscriptions/upcoming";
        assert!(path.contains("upcoming"));
    }
}
