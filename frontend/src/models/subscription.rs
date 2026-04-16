use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub start_date: NaiveDate,
    pub next_billing_date: Option<NaiveDate>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateSubscriptionRequest {
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub start_date: NaiveDate,
    pub category_id: Option<Uuid>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateSubscriptionRequest {
    pub name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub frequency: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub next_billing_date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalendarItem {
    pub subscription_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub billing_date: NaiveDate,
    pub is_active: bool,
    pub logo_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_subscription() -> Subscription {
        Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Netflix".into(),
            amount: dec!(15.99),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: Some(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()),
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn subscription_serialization_roundtrip() {
        let sub = sample_subscription();
        let json = serde_json::to_string(&sub).unwrap();
        let deserialized: Subscription = serde_json::from_str(&json).unwrap();
        assert_eq!(sub, deserialized);
    }

    #[test]
    fn subscription_fields_match() {
        let sub = sample_subscription();
        assert_eq!(sub.name, "Netflix");
        assert_eq!(sub.amount, dec!(15.99));
        assert_eq!(sub.currency, "EUR");
        assert_eq!(sub.frequency, "monthly");
        assert!(sub.is_active);
    }

    #[test]
    fn create_request_serialization() {
        let req = CreateSubscriptionRequest {
            name: "Spotify".into(),
            amount: dec!(9.99),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            category_id: None,
            notes: Some("Music streaming".into()),
            logo_url: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Spotify"));
        let back: CreateSubscriptionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn calendar_item_serialization() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Netflix".into(),
            amount: dec!(15.99),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            billing_date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            is_active: true,
            logo_url: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: CalendarItem = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Netflix");
        assert_eq!(back.billing_date, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    }

    #[test]
    fn update_request_default_all_none() {
        let req = UpdateSubscriptionRequest::default();
        assert!(req.name.is_none());
        assert!(req.amount.is_none());
        assert!(req.currency.is_none());
        assert!(req.frequency.is_none());
        assert!(req.start_date.is_none());
        assert!(req.next_billing_date.is_none());
        assert!(req.category_id.is_none());
        assert!(req.is_active.is_none());
        assert!(req.notes.is_none());
        assert!(req.logo_url.is_none());
    }

    #[test]
    fn update_request_partial_fields() {
        let req = UpdateSubscriptionRequest {
            name: Some("New Name".into()),
            amount: Some(dec!(20)),
            is_active: Some(false),
            ..Default::default()
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("New Name"));
        let back: UpdateSubscriptionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, Some("New Name".into()));
        assert_eq!(back.amount, Some(dec!(20)));
        assert_eq!(back.is_active, Some(false));
        assert!(back.frequency.is_none());
    }

    #[test]
    fn subscription_inactive_fields() {
        let mut sub = sample_subscription();
        sub.is_active = false;
        sub.notes = Some("Cancelled".into());
        sub.logo_url = Some("https://logo.com/n.png".into());
        assert!(!sub.is_active);
        assert_eq!(sub.notes.as_deref(), Some("Cancelled"));
        assert!(sub.logo_url.is_some());
    }

    #[test]
    fn subscription_with_no_next_billing() {
        let mut sub = sample_subscription();
        sub.next_billing_date = None;
        assert!(sub.next_billing_date.is_none());
    }

    #[test]
    fn create_request_with_category() {
        let req = CreateSubscriptionRequest {
            name: "Hulu".into(),
            amount: dec!(12.99),
            currency: "USD".into(),
            frequency: "monthly".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            category_id: Some(Uuid::new_v4()),
            notes: None,
            logo_url: Some("https://logo.com/hulu.png".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("hulu.png"));
        let back: CreateSubscriptionRequest = serde_json::from_str(&json).unwrap();
        assert!(back.category_id.is_some());
        assert!(back.logo_url.is_some());
    }

    #[test]
    fn update_request_is_active_toggle() {
        let req = UpdateSubscriptionRequest {
            is_active: Some(false),
            ..Default::default()
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("false"));
        let back: UpdateSubscriptionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.is_active, Some(false));
        assert!(back.name.is_none());
    }

    #[test]
    fn update_request_with_notes() {
        let req = UpdateSubscriptionRequest {
            notes: Some("Updated notes".into()),
            ..Default::default()
        };
        assert_eq!(req.notes.as_deref(), Some("Updated notes"));
    }

    #[test]
    fn calendar_item_fields() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Test".into(),
            amount: dec!(9.99),
            currency: "USD".into(),
            frequency: "weekly".into(),
            billing_date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            is_active: true,
            logo_url: None,
        };
        assert_eq!(item.frequency, "weekly");
        assert!(item.is_active);
        assert!(item.logo_url.is_none());
    }

    #[test]
    fn subscription_equality() {
        let sub = sample_subscription();
        let sub2 = sub.clone();
        assert_eq!(sub, sub2);
    }
}