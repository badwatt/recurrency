#![allow(dead_code)]

//! Test utilities for handler unit testing.

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::auth::models::{Claims, UserResponse};
use crate::categories::models::CategoryResponse;
use crate::categories::service::CategoryWithCounts;
use crate::subscriptions::models::SubscriptionResponse;
use crate::subscriptions::service::CalendarItem;
use crate::transactions::models::TransactionResponse;

// ============================================================================
// Mock Data Builders
// ============================================================================

/// Create a mock Claims for testing.
pub fn create_mock_claims(user_id: Option<Uuid>, email: Option<&str>) -> Claims {
    let uid = user_id.unwrap_or_else(Uuid::new_v4);
    let exp = (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let iat = Utc::now().timestamp() as usize;
    Claims {
        sub: uid,
        email: email.unwrap_or("test@example.com").to_string(),
        exp,
        iat,
    }
}

/// Create a mock JWT token for testing.
pub fn create_mock_token(user_id: Uuid, email: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::sync::OnceLock;

    static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();
    let secret = JWT_SECRET.get_or_init(|| {
        std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "super_secret_jwt_key_change_in_production".to_string())
            .into_bytes()
    });

    let claims = create_mock_claims(Some(user_id), Some(email));
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_slice()),
    )
    .unwrap()
}

/// Create a mock CategoryResponse.
pub fn create_mock_category(name: &str) -> CategoryResponse {
    let now = Utc::now();
    let uid = Uuid::new_v4();
    CategoryResponse {
        id: uid,
        user_id: uid,
        name: name.to_string(),
        icon: Some("star".to_string()),
        color: Some("#ff0000".to_string()),
        is_system: false,
        is_default: false,
        created_at: now,
        updated_at: now,
    }
}

/// Create a mock CategoryWithCounts.
pub fn create_mock_category_with_counts(name: &str) -> CategoryWithCounts {
    CategoryWithCounts {
        category: create_mock_category(name),
        subscription_count: 5,
        transaction_count: 10,
    }
}

/// Create a mock SubscriptionResponse.
pub fn create_mock_subscription(
    name: &str,
    amount: Option<Decimal>,
    currency: Option<&str>,
) -> SubscriptionResponse {
    let now = Utc::now();
    let uid = Uuid::new_v4();
    SubscriptionResponse {
        id: uid,
        user_id: uid,
        category_id: uid,
        name: name.to_string(),
        amount: amount.unwrap_or(Decimal::new(999, 2)),
        currency: currency.unwrap_or("USD").to_string(),
        frequency: "monthly".to_string(),
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        next_billing_date: Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
        is_active: true,
        notes: None,
        logo_url: None,
        created_at: now,
        updated_at: now,
    }
}

/// Create a mock TransactionResponse.
pub fn create_mock_transaction(name: &str, amount: Option<Decimal>) -> TransactionResponse {
    let now = Utc::now();
    let uid = Uuid::new_v4();
    TransactionResponse {
        id: uid,
        user_id: uid,
        category_id: uid,
        name: name.to_string(),
        amount: amount.unwrap_or(Decimal::new(4999, 2)),
        currency: "USD".to_string(),
        date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        notes: None,
        logo_url: None,
        created_at: now,
        updated_at: now,
    }
}

/// Create a mock CalendarItem.
pub fn create_mock_calendar_item(name: &str) -> CalendarItem {
    CalendarItem {
        subscription_id: Uuid::new_v4(),
        name: name.to_string(),
        amount: Decimal::new(999, 2),
        currency: "USD".to_string(),
        frequency: "monthly".to_string(),
        billing_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        is_active: true,
        logo_url: None,
    }
}

/// Create a mock UserResponse.
pub fn create_mock_user(email: Option<&str>) -> UserResponse {
    UserResponse {
        id: Uuid::new_v4(),
        email: email.unwrap_or("test@example.com").to_string(),
        name: Some("Test User".to_string()),
        currency: "USD".to_string(),
        verified: true,
    }
}

// ============================================================================
// JSON Serialization Tests
// ============================================================================

#[cfg(test)]
mod subscription_response_tests {
    use super::*;

    #[test]
    fn subscription_response_serialization() {
        let sub = create_mock_subscription("Netflix", None, None);
        let json = serde_json::to_value(&sub).unwrap();
        assert_eq!(json["name"], "Netflix");
        assert!(json["is_active"].as_bool().unwrap());
    }

    #[test]
    fn subscription_response_with_notes_and_logo() {
        let now = Utc::now();
        let uid = Uuid::new_v4();
        let sub = SubscriptionResponse {
            id: uid,
            user_id: uid,
            category_id: uid,
            name: "Test".to_string(),
            amount: Decimal::ONE,
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            is_active: true,
            notes: Some("Test notes".to_string()),
            logo_url: Some("https://logo.com/test.png".to_string()),
            created_at: now,
            updated_at: now,
        };

        let json = serde_json::to_value(&sub).unwrap();
        assert_eq!(json["notes"], "Test notes");
        assert_eq!(json["logo_url"], "https://logo.com/test.png");
    }
}

#[cfg(test)]
mod transaction_response_tests {
    use super::*;

    #[test]
    fn transaction_response_serialization() {
        let tx = create_mock_transaction("Grocery Store", None);
        let json = serde_json::to_value(&tx).unwrap();
        assert_eq!(json["name"], "Grocery Store");
        assert!(json.get("date").is_some());
    }

    #[test]
    fn transaction_response_date_format() {
        let tx = create_mock_transaction("Coffee", Some(Decimal::new(499, 2)));
        let json = serde_json::to_value(&tx).unwrap();
        let date_str = json["date"].as_str().unwrap();
        assert!(date_str.starts_with("2024"));
    }
}

#[cfg(test)]
mod category_response_tests {
    use super::*;

    #[test]
    fn category_response_serialization() {
        let cat = create_mock_category("Entertainment");
        let json = serde_json::to_value(&cat).unwrap();
        assert_eq!(json["name"], "Entertainment");
        assert!(json.get("icon").is_some());
        assert!(json.get("color").is_some());
    }

    #[test]
    fn category_with_counts_serialization() {
        let cat = create_mock_category_with_counts("Food");
        let json = serde_json::to_value(&cat).unwrap();
        assert_eq!(json["category"]["name"], "Food");
        assert_eq!(json["subscription_count"], 5);
        assert_eq!(json["transaction_count"], 10);
    }
}

#[cfg(test)]
mod calendar_item_tests {
    use super::*;

    #[test]
    fn calendar_item_serialization() {
        let item = create_mock_calendar_item("Netflix");
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["name"], "Netflix");
        assert!(json.get("billing_date").is_some());
    }

    #[test]
    fn calendar_item_has_billing_date_not_date() {
        let item = create_mock_calendar_item("Test");
        let json = serde_json::to_value(&item).unwrap();
        assert!(json.get("billing_date").is_some());
        assert!(json.get("date").is_none());
    }
}

#[cfg(test)]
mod user_response_tests {
    use super::*;

    #[test]
    fn user_response_serialization() {
        let user = create_mock_user(Some("alice@example.com"));
        let json = serde_json::to_value(&user).unwrap();
        assert_eq!(json["email"], "alice@example.com");
        assert_eq!(json["name"], "Test User");
        assert!(json["verified"].as_bool().unwrap());
    }

    #[test]
    fn user_response_excludes_password_hash() {
        let json = serde_json::to_value(&create_mock_user(None)).unwrap();
        assert!(json.get("password_hash").is_none());
    }
}

#[cfg(test)]
mod claims_tests {
    use super::*;

    #[test]
    fn create_mock_claims_generates_valid_claims() {
        let claims = create_mock_claims(None, None);
        assert!(claims.exp > claims.iat);
        assert!(!claims.email.is_empty());
    }

    #[test]
    fn create_mock_token_produces_valid_jwt() {
        let user_id = Uuid::new_v4();
        let token = create_mock_token(user_id, "test@test.com");
        assert!(!token.is_empty());
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }
}

#[cfg(test)]
mod calendar_frequency_tests {
    use super::*;

    fn create_calendar_item_with_frequency(frequency: &str) -> CalendarItem {
        CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Test Sub".to_string(),
            amount: Decimal::TEN,
            currency: "USD".to_string(),
            frequency: frequency.to_string(),
            billing_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            is_active: true,
            logo_url: None,
        }
    }

    #[test]
    fn weekly_frequency_serialization() {
        let item = create_calendar_item_with_frequency("weekly");
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["frequency"], "weekly");
    }

    #[test]
    fn monthly_frequency_serialization() {
        let item = create_calendar_item_with_frequency("monthly");
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["frequency"], "monthly");
    }

    #[test]
    fn quarterly_frequency_serialization() {
        let item = create_calendar_item_with_frequency("quarterly");
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["frequency"], "quarterly");
    }

    #[test]
    fn yearly_frequency_serialization() {
        let item = create_calendar_item_with_frequency("yearly");
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["frequency"], "yearly");
    }
}
