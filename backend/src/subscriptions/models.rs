use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubscriptionRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    #[validate(length(min = 1, message = "Frequency is required"))]
    pub frequency: String, // weekly, monthly, quarterly, yearly
    pub start_date: NaiveDate,
    pub category_id: Option<Uuid>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
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

impl From<Subscription> for SubscriptionResponse {
    fn from(s: Subscription) -> Self {
        Self {
            id: s.id,
            user_id: s.user_id,
            category_id: s.category_id,
            name: s.name,
            amount: s.amount,
            currency: s.currency,
            frequency: s.frequency,
            start_date: s.start_date,
            next_billing_date: s.next_billing_date,
            is_active: s.is_active,
            notes: s.notes,
            logo_url: s.logo_url,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn create_subscription_request_valid() {
        let req = CreateSubscriptionRequest {
            name: "Netflix".to_string(),
            amount: Decimal::from(1599),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            category_id: None,
            notes: None,
            logo_url: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_subscription_request_empty_name_fails() {
        let req = CreateSubscriptionRequest {
            name: "".to_string(),
            amount: Decimal::TEN,
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            category_id: None,
            notes: None,
            logo_url: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_subscription_request_empty_frequency_fails() {
        let req = CreateSubscriptionRequest {
            name: "Netflix".to_string(),
            amount: Decimal::TEN,
            currency: "USD".to_string(),
            frequency: "".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            category_id: None,
            notes: None,
            logo_url: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn update_subscription_all_none() {
        let req = UpdateSubscriptionRequest {
            name: None,
            amount: None,
            currency: None,
            frequency: None,
            start_date: None,
            next_billing_date: None,
            category_id: None,
            is_active: None,
            notes: None,
            logo_url: None,
        };
        assert!(req.name.is_none());
        assert!(req.amount.is_none());
        assert!(req.is_active.is_none());
    }

    #[test]
    fn update_subscription_some_fields() {
        let req = UpdateSubscriptionRequest {
            name: Some("New Name".to_string()),
            amount: Some(Decimal::TEN),
            currency: None,
            frequency: Some("yearly".to_string()),
            start_date: None,
            next_billing_date: Some(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
            category_id: None,
            is_active: Some(false),
            notes: None,
            logo_url: None,
        };
        assert_eq!(req.name.as_deref(), Some("New Name"));
        assert_eq!(req.frequency.as_deref(), Some("yearly"));
        assert_eq!(req.is_active, Some(false));
    }

    #[test]
    fn subscription_response_from_subscription() {
        let uid = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let cat_id = Uuid::new_v4();
        let sub = Subscription {
            id: uid,
            user_id,
            category_id: cat_id,
            name: "Test".to_string(),
            amount: Decimal::TEN,
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: Some(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()),
            is_active: true,
            notes: Some("notes".to_string()),
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = SubscriptionResponse::from(sub);
        assert_eq!(resp.id, uid);
        assert_eq!(resp.name, "Test");
        assert!(resp.is_active);
    }

    #[test]
    fn subscription_response_serialization_fields() {
        let uid = Uuid::new_v4();
        let resp = SubscriptionResponse {
            id: uid,
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Netflix".to_string(),
            amount: Decimal::from(1599),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("id").is_some());
        assert!(json.get("name").is_some());
        assert!(json.get("amount").is_some());
        assert!(json.get("frequency").is_some());
    }

    #[test]
    fn create_subscription_all_frequencies() {
        for freq in &["weekly", "monthly", "quarterly", "yearly"] {
            let req = CreateSubscriptionRequest {
                name: "Test".to_string(),
                amount: Decimal::TEN,
                currency: "USD".to_string(),
                frequency: freq.to_string(),
                start_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                category_id: None,
                notes: None,
                logo_url: None,
            };
            assert!(req.validate().is_ok());
        }
    }
}
