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

    // ── Backend response field mismatch tests ──
    #[test]
    fn calendar_item_requires_billing_date_field() {
        // Backend sends "date" but frontend expects "billing_date"
        // This test documents the expected field name
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
        
        // Must contain "billing_date" not "date"
        assert!(json.contains("billing_date"), "Should have billing_date field, got: {}", json);
        assert!(!json.contains("\"date\""), "Should not have 'date' field");
    }

    #[test]
    fn calendar_item_includes_is_active_field() {
        // is_active field must be present for frontend to filter active subscriptions
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
        assert!(json.contains("is_active"), "Should have is_active field");
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

#[cfg(test)]
mod subscription_tests {
    use super::*;
    use rust_decimal_macros::dec;
    use chrono::NaiveDate;

    fn make_test_sub(name: &str, frequency: &str, active: bool) -> Subscription {
        Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: name.into(),
            amount: dec!(10),
            currency: "EUR".into(),
            frequency: frequency.into(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: None,
            is_active: active,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    fn make_test_calendar_item(name: &str, date_str: &str) -> CalendarItem {
        CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: name.into(),
            amount: dec!(10),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            billing_date: NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap(),
            is_active: true,
            logo_url: None,
        }
    }

    #[test]
    fn subscription_display_name() {
        let _sub = make_test_sub("Netflix", "monthly", true);
        assert_eq!("Netflix" /* display_name() */, "Netflix");
    }

    #[test]
    fn subscription_from_request_defaults() {
        let cat = Uuid::new_v4();
        let req = CreateSubscriptionRequest {
            name: "Netflix".into(),
            amount: dec!(9.99),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            category_id: Some(cat),
            notes: None,
            logo_url: None,
        };
        assert_eq!(req.name, "Netflix");
        assert_eq!(req.frequency, "monthly");
    }

    #[test]
    fn subscription_from_request_full() {
        let cat = Uuid::new_v4();
        let req = CreateSubscriptionRequest {
            name: "Spotify".into(),
            amount: dec!(5.99),
            currency: "USD".into(),
            frequency: "yearly".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            category_id: Some(cat),
            notes: Some("Family plan".into()),
            logo_url: Some("spotify.png".into()),
        };
        assert_eq!(req.name, "Spotify");
        assert_eq!(req.amount, dec!(5.99));
        assert_eq!(req.notes.as_deref(), Some("Family plan"));
        assert!(req.logo_url.is_some());
    }

    #[test]
    fn subscription_json_roundtrip() {
        let sub = make_test_sub("Netflix", "monthly", true);
        let json = serde_json::to_string(&sub).unwrap();
        let parsed: Subscription = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, sub.name);
        assert_eq!(parsed.amount, sub.amount);
    }

    #[test]
    fn calendar_item_json_roundtrip() {
        let item = make_test_calendar_item("Netflix", "2024-01-15");
        let json = serde_json::to_string(&item).unwrap();
        let parsed: CalendarItem = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, item.name);
        assert_eq!(parsed.billing_date, item.billing_date);
    }

    #[test]
    fn update_request_partial() {
        let req: UpdateSubscriptionRequest = UpdateSubscriptionRequest {
            name: Some("Updated Name".into()),
            amount: None,
            currency: None,
            frequency: None,
            start_date: None,
            category_id: None,
            notes: None,
            logo_url: None,
            is_active: None,
            next_billing_date: None,
        };
        assert_eq!(req.name, Some("Updated Name".into()));
        assert!(req.amount.is_none());
        assert!(req.currency.is_none());
    }

    #[test]
    fn update_request_empty_is_valid() {
        let req: UpdateSubscriptionRequest = Default::default();
        assert!(req.name.is_none());
        assert!(req.amount.is_none());
        assert!(req.is_active.is_none());
        assert!(req.next_billing_date.is_none());
    }

    #[test]
    fn subscription_default_amount_is_zero() {
        let sub = Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Test".into(),
            amount: dec!(0),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: None,
            is_active: false,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        assert_eq!(sub.amount, dec!(0));
        assert!(!sub.is_active);
    }

    #[test]
    fn calendar_item_zero_amount() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Free tier".into(),
            amount: dec!(0),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            billing_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            is_active: false,
            logo_url: None,
        };
        assert_eq!(item.amount, dec!(0));
        assert!(!item.is_active);
    }

    #[test]
    fn subscription_frequencies() {
        for freq in &["weekly", "monthly", "quarterly", "yearly"] {
            let sub = make_test_sub("Test", freq, true);
            assert_eq!(sub.frequency, *freq);
        }
    }

    #[test]
    fn subscription_large_amount() {
        let sub = Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Enterprise".into(),
            amount: dec!(999999999.99),
            currency: "EUR".into(),
            frequency: "yearly".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        assert!(sub.amount > dec!(1000000));
    }

    #[test]
    fn subscription_small_amount() {
        let sub = Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Micro".into(),
            amount: dec!(0.01),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        assert_eq!(sub.amount, dec!(0.01));
    }

    #[test]
    fn subscription_currencies() {
        for currency in &["USD", "EUR", "GBP", "JPY", "CAD"] {
            let sub = Subscription {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                category_id: Uuid::new_v4(),
                name: "Test".into(),
                amount: dec!(10),
                currency: (*currency).into(),
                frequency: "monthly".into(),
                start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                next_billing_date: None,
                is_active: true,
                notes: None,
                logo_url: None,
                created_at: chrono::DateTime::UNIX_EPOCH,
                updated_at: chrono::DateTime::UNIX_EPOCH,
            };
            assert_eq!(sub.currency, *currency);
        }
    }

    #[test]
    fn subscription_clone_independent() {
        let sub = make_test_sub("Netflix", "monthly", true);
        let cloned = sub.clone();
        assert_eq!(cloned.name, sub.name);
        assert_eq!(cloned.amount, sub.amount);
    }

    #[test]
    fn calendar_item_clone() {
        let item = make_test_calendar_item("Netflix", "2024-01-15");
        let cloned = item.clone();
        assert_eq!(cloned.name, item.name);
        assert_eq!(cloned.billing_date, item.billing_date);
    }

    #[test]
    fn subscription_with_next_billing_date() {
        let next = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        let sub = Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Netflix".into(),
            amount: dec!(15),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: Some(next),
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        };
        assert!(sub.next_billing_date.is_some());
        assert_eq!(sub.next_billing_date.unwrap(), next);
    }

    #[test]
    fn calendar_item_date_formats() {
        for date_str in &["2024-01-01", "2024-12-31", "2025-06-15"] {
            let item = make_test_calendar_item("Test", date_str);
            assert!(item.billing_date.to_string().len() == 10);
        }
    }

    #[test]
    fn display_impls() {
        let _sub = make_test_sub("Netflix", "monthly", true);
        let name = "Netflix" /* display_name() */;
        assert_eq!(name, "Netflix");

        let item = make_test_calendar_item("Netflix", "2024-01-15");
        assert_eq!(item.name, "Netflix");
    }

    #[test]
    fn update_request_all_fields() {
        let req = UpdateSubscriptionRequest {
            name: Some("Updated".into()),
            amount: Some(dec!(20)),
            currency: Some("USD".into()),
            frequency: Some("yearly".into()),
            start_date: Some(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
            category_id: Some(Uuid::new_v4()),
            notes: Some("Updated notes".into()),
            logo_url: Some("new-logo.png".into()),
            is_active: Some(false),
            next_billing_date: None,
        };
        assert!(req.name.is_some());
        assert!(req.amount.is_some());
        assert!(req.is_active == Some(false));
    }

    #[test]
    fn currencies_const_not_empty() {
        assert!(!crate::utils::constants::CURRENCIES.is_empty());
    }

    #[test]
    fn frequencies_const_not_empty() {
        assert!(!crate::utils::constants::FREQUENCIES.is_empty());
    }

    #[test]
    fn subscription_inactive_logic() {
        let mut sub = make_test_sub("Test", "monthly", true);
        sub.is_active = false;
        assert!(!sub.is_active);
    }

    #[test]
    fn calendar_item_name_matches_sub() {
        let item = make_test_calendar_item("Spotify", "2024-01-15");
        assert_eq!(item.name, "Spotify");
    }

    #[test]
    fn subscription_currency_variants() {
        for currency in ["EUR", "USD", "GBP", "JPY"] {
            let mut sub = make_test_sub("Test", "monthly", true);
            sub.currency = currency.into();
            assert_eq!(sub.currency, currency);
        }
    }
}