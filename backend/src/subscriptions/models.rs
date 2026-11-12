use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;
use uuid::Uuid;
use validator::Validate;


/// Unified recurrence definition stored as JSONB.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recurrence {
    pub interval: u16,
    /// "day", "week", "month" or "year".
    pub unit: String,
    /// Days of the month. 0 means the last day of the month.
    #[serde(default)]
    pub days: Vec<u8>,
    /// ISO weekdays, Monday = 1.
    #[serde(default)]
    pub weekdays: Vec<u8>,
    /// Explicit annual dates, optionally with a per-date amount.
    #[serde(default)]
    pub dates: Vec<AnnualDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnnualDate {
    pub month: u8,
    pub day: u8,
    #[serde(default)]
    pub amount: Option<Decimal>,
}

impl Default for Recurrence {
    fn default() -> Self {
        Self {
            interval: 1,
            unit: "month".to_string(),
            days: vec![1],
            weekdays: Vec::new(),
            dates: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub schedule_day: Option<i16>,
    pub schedule_month: Option<i16>,
    pub recurrence: Json<Recurrence>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub next_billing_date: Option<NaiveDate>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateSubscriptionRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    #[validate(length(min = 1, message = "Frequency is required"))]
    pub frequency: String, // daily, weekly, monthly, quarterly, yearly
    #[serde(default)]
    pub schedule_day: Option<i16>,
    #[serde(default)]
    pub schedule_month: Option<i16>,
    #[serde(default)]
    pub recurrence: Option<Recurrence>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default = "today_date")]
    pub start_date: NaiveDate,
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    #[serde(default)]
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub frequency: Option<String>,
    pub schedule_day: Option<i16>,
    pub schedule_month: Option<i16>,
    #[serde(default)]
    pub recurrence: Option<Recurrence>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    #[serde(default)]
    pub clear_end_date: bool,
    pub next_billing_date: Option<NaiveDate>,
    pub account_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub schedule_day: Option<i16>,
    pub schedule_month: Option<i16>,
    pub recurrence: Recurrence,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
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
            account_id: s.account_id,
            category_id: s.category_id,
            name: s.name,
            amount: s.amount,
            currency: s.currency,
            frequency: s.frequency,
            schedule_day: s.schedule_day,
            schedule_month: s.schedule_month,
            recurrence: s.recurrence.0,
            start_date: s.start_date,
            end_date: s.end_date,
            next_billing_date: s.next_billing_date,
            is_active: s.is_active,
            notes: s.notes,
            logo_url: s.logo_url,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

/// When a client omits the start date, the recurrence anchors on today.
fn today_date() -> NaiveDate {
    Utc::now().date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn create_request_without_start_date_defaults_to_today() {
        let req: CreateSubscriptionRequest = serde_json::from_str(
            r#"{"name":"Started today","amount":"10.00","currency":"EUR","frequency":"monthly"}"#,
        )
        .unwrap();
        assert_eq!(req.start_date, Utc::now().date_naive());
    }

    #[test]
    fn create_subscription_request_valid() {
        let req = CreateSubscriptionRequest {
            account_id: Uuid::new_v4(),
            name: "Netflix".to_string(),
            amount: Decimal::from(1599),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            is_active: None,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            end_date: None,
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
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            is_active: None,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            end_date: None,
            account_id: Uuid::new_v4(),
            category_id: None,
            notes: None,
            logo_url: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_subscription_request_empty_frequency_fails() {
        let req = CreateSubscriptionRequest {
            account_id: Uuid::new_v4(),
            name: "Netflix".to_string(),
            amount: Decimal::TEN,
            currency: "USD".to_string(),
            frequency: "".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            is_active: None,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            end_date: None,
            category_id: None,
            notes: None,
            logo_url: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn update_subscription_all_none() {
        let req = UpdateSubscriptionRequest {
            account_id: None,
            name: None,
            amount: None,
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
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            start_date: None,
            end_date: None,
            clear_end_date: false,
            next_billing_date: Some(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
            account_id: None,
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
            account_id: Uuid::new_v4(),
            id: uid,
            user_id,
            category_id: cat_id,
            name: "Test".to_string(),
            amount: Decimal::TEN,
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: sqlx::types::Json(crate::subscriptions::models::Recurrence::default()),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
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
            account_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Netflix".to_string(),
            amount: Decimal::from(1599),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: Recurrence::default(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
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
                account_id: Uuid::new_v4(),
                name: "Test".to_string(),
                amount: Decimal::TEN,
                currency: "USD".to_string(),
                frequency: freq.to_string(),
                schedule_day: None,
                schedule_month: None,
                recurrence: None,
                is_active: None,
                start_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                end_date: None,
                category_id: None,
                notes: None,
                logo_url: None,
            };
            assert!(req.validate().is_ok());
        }
    }

    #[test]
    fn subscription_with_logo_url_roundtrip() {
        let resp = SubscriptionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Netflix".to_string(),
            amount: Decimal::from(1599),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: Recurrence::default(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: Some("https://logo.clearbit.com/netflix.com".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("logo_url").is_some());
        assert!(json["logo_url"].is_string());
    }

    #[test]
    fn subscription_inactive_status() {
        let resp = SubscriptionResponse {
            account_id: Uuid::new_v4(),
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Cancelled".to_string(),
            amount: Decimal::from(999),
            currency: "EUR".to_string(),
            frequency: "monthly".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: Recurrence::default(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            end_date: None,
            next_billing_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            is_active: false,
            notes: Some("Cancelled in December".to_string()),
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(!resp.is_active);
        assert!(resp.notes.is_some());
    }

    #[test]
    fn create_subscription_with_category_id() {
        let cat_id = Uuid::new_v4();
        let req = CreateSubscriptionRequest {
            name: "Spotify".to_string(),
            amount: Decimal::from(999),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            is_active: None,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            end_date: None,
            account_id: Uuid::new_v4(),
            category_id: Some(cat_id),
            notes: None,
            logo_url: None,
        };
        assert!(req.validate().is_ok());
        assert_eq!(req.category_id, Some(cat_id));
    }

    #[test]
    fn update_subscription_with_next_billing_date() {
        let req = UpdateSubscriptionRequest {
            account_id: None,
            name: None,
            amount: None,
            currency: None,
            frequency: None,
            schedule_day: None,
            schedule_month: None,
            recurrence: None,
            start_date: None,
            end_date: None,
            clear_end_date: false,
            next_billing_date: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            category_id: None,
            is_active: Some(true),
            notes: None,
            logo_url: None,
        };
        assert!(req.next_billing_date.is_some());
        assert_eq!(req.is_active, Some(true));
    }

    #[test]
    fn subscription_response_next_billing_date_some() {
        let resp = SubscriptionResponse {
            account_id: Uuid::new_v4(),
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Active Sub".to_string(),
            amount: Decimal::TEN,
            currency: "GBP".to_string(),
            frequency: "weekly".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: Recurrence::default(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            next_billing_date: Some(NaiveDate::from_ymd_opt(2024, 6, 20).unwrap()),
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(resp.next_billing_date.is_some());
    }

    #[test]
    fn subscription_response_next_billing_date_none() {
        let resp = SubscriptionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "No Next Date".to_string(),
            amount: Decimal::ONE,
            currency: "JPY".to_string(),
            frequency: "yearly".to_string(),
            schedule_day: None,
            schedule_month: None,
            recurrence: Recurrence::default(),
            start_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            end_date: None,
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(resp.next_billing_date.is_none());
    }
}
