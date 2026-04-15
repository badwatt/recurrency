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
}