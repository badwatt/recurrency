//! Shared subscription form types, validation, and initialization.
//!
//! Used by both SubscriptionsPage and CalendarPage to avoid duplication.

use rust_decimal::Decimal;
use uuid::Uuid;

/// Shared form state for subscription create/edit dialogs.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionForm {
    pub name: String,
    pub amount: String,
    pub currency: String,
    pub frequency: String,
    pub start_date: String,
    pub category_id: String,
    pub notes: String,
    pub logo_url: String,
}

impl SubscriptionForm {
    /// Reset form to default values.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Initialize form for creating a new subscription.
    pub fn init_for_create(date: &str, currency: &str, category_id: &str) -> Self {
        Self {
            name: String::new(),
            amount: String::new(),
            currency: currency.to_string(),
            frequency: "monthly".to_string(),
            start_date: date.to_string(),
            category_id: category_id.to_string(),
            notes: String::new(),
            logo_url: String::new(),
        }
    }

    /// Initialize form from an existing subscription (for editing).
    pub fn from_subscription(
        name: &str,
        amount: &str,
        currency: &str,
        frequency: &str,
        start_date: &str,
        category_id: &str,
        notes: &str,
        logo_url: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            amount: amount.to_string(),
            currency: currency.to_string(),
            frequency: frequency.to_string(),
            start_date: start_date.to_string(),
            category_id: category_id.to_string(),
            notes: notes.to_string(),
            logo_url: logo_url.to_string(),
        }
    }

    /// Parse the amount field as Decimal, returning None if invalid.
    pub fn parse_amount(&self) -> Option<Decimal> {
        self.amount.parse::<Decimal>().ok()
    }

    /// Parse the start_date field as NaiveDate, returning None if invalid.
    pub fn parse_start_date(&self) -> Option<chrono::NaiveDate> {
        self.start_date.parse::<chrono::NaiveDate>().ok()
    }

    /// Parse the category_id field as Uuid, returning None if invalid.
    pub fn parse_category_id(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.category_id).ok()
    }

    /// Build an UpdateSubscriptionRequest from the form fields.
    /// Returns None if amount cannot be parsed.
    pub fn to_update_request(&self) -> Option<crate::models::subscription::UpdateSubscriptionRequest> {
        let amount = self.parse_amount()?;
        let category = self.parse_category_id();
        Some(crate::models::subscription::UpdateSubscriptionRequest {
            name: Some(self.name.clone()),
            amount: Some(amount),
            currency: Some(self.currency.clone()),
            frequency: Some(self.frequency.clone()),
            start_date: Some(self.parse_start_date().unwrap_or(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).expect("2024-01-01 is a valid date"),
            )),
            category_id: category,
            notes: if self.notes.is_empty() { None } else { Some(self.notes.clone()) },
            logo_url: if self.logo_url.is_empty() { None } else { Some(self.logo_url.clone()) },
            ..Default::default()
        })
    }

    /// Build a CreateSubscriptionRequest from the form fields.
    /// Returns None if amount or date cannot be parsed.
    pub fn to_create_request(&self) -> Option<crate::models::subscription::CreateSubscriptionRequest> {
        let amount = self.parse_amount()?;
        let category = self.parse_category_id();
        Some(crate::models::subscription::CreateSubscriptionRequest {
            name: self.name.clone(),
            amount,
            currency: self.currency.clone(),
            frequency: self.frequency.clone(),
            start_date: self.parse_start_date().unwrap_or(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).expect("2024-01-01 is a valid date"),
            ),
            category_id: category,
            notes: if self.notes.is_empty() { None } else { Some(self.notes.clone()) },
            logo_url: if self.logo_url.is_empty() { None } else { Some(self.logo_url.clone()) },
        })
    }
}

/// Validation result type.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ValidationResult {
    pub name_error: Option<String>,
    pub amount_error: Option<String>,
    pub currency_error: Option<String>,
    pub frequency_error: Option<String>,
    pub start_date_error: Option<String>,
    pub category_error: Option<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.name_error.is_none()
            && self.amount_error.is_none()
            && self.currency_error.is_none()
            && self.frequency_error.is_none()
            && self.start_date_error.is_none()
            && self.category_error.is_none()
    }
}

/// Pure validation function for subscription form.
pub fn validate_subscription_form(
    name: &str,
    amount: &str,
    currency: &str,
    frequency: &str,
    start_date: &str,
    category_id: &str,
) -> ValidationResult {
    let mut result = ValidationResult::default();
    if name.trim().is_empty() {
        result.name_error = Some("Name is required".into());
    } else if name.trim().len() < 2 {
        result.name_error = Some("Name must be at least 2 characters".into());
    }
    if amount.is_empty() {
        result.amount_error = Some("Amount is required".into());
    } else {
        match amount.parse::<Decimal>() {
            Ok(a) if a <= Decimal::ZERO => {
                result.amount_error = Some("Amount must be greater than 0".into());
            }
            Err(_) => {
                result.amount_error = Some("Invalid amount format".into());
            }
            _ => {}
        }
    }
    if currency.is_empty() {
        result.currency_error = Some("Currency is required".into());
    }
    if frequency.is_empty() {
        result.frequency_error = Some("Frequency is required".into());
    }
    if start_date.is_empty() {
        result.start_date_error = Some("Start date is required".into());
    } else if start_date.parse::<chrono::NaiveDate>().is_err() {
        result.start_date_error = Some("Invalid date format".into());
    }
    if category_id.is_empty() {
        result.category_error = Some("Category is required".into());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── SubscriptionForm tests ──

    #[test]
    fn form_default_has_empty_fields() {
        let form = SubscriptionForm::default();
        assert!(form.name.is_empty());
        assert!(form.amount.is_empty());
        assert!(form.currency.is_empty());
        assert!(form.frequency.is_empty());
        assert!(form.start_date.is_empty());
        assert!(form.category_id.is_empty());
        assert!(form.notes.is_empty());
        assert!(form.logo_url.is_empty());
    }

    #[test]
    fn form_init_for_create_sets_defaults() {
        let form = SubscriptionForm::init_for_create("2024-06-15", "USD", "cat-123");
        assert!(form.name.is_empty());
        assert!(form.amount.is_empty());
        assert_eq!(form.currency, "USD");
        assert_eq!(form.frequency, "monthly");
        assert_eq!(form.start_date, "2024-06-15");
        assert_eq!(form.category_id, "cat-123");
        assert!(form.notes.is_empty());
        assert!(form.logo_url.is_empty());
    }

    #[test]
    fn form_from_subscription_populates_all_fields() {
        let form = SubscriptionForm::from_subscription(
            "Netflix", "9.99", "EUR", "monthly", "2024-01-01", "cat-456", "my notes", "logo.png",
        );
        assert_eq!(form.name, "Netflix");
        assert_eq!(form.amount, "9.99");
        assert_eq!(form.currency, "EUR");
        assert_eq!(form.frequency, "monthly");
        assert_eq!(form.start_date, "2024-01-01");
        assert_eq!(form.category_id, "cat-456");
        assert_eq!(form.notes, "my notes");
        assert_eq!(form.logo_url, "logo.png");
    }

    #[test]
    fn form_reset_clears_all_fields() {
        let mut form = SubscriptionForm::from_subscription(
            "Netflix", "9.99", "EUR", "monthly", "2024-01-01", "cat-456", "notes", "logo.png",
        );
        form.reset();
        assert!(form.name.is_empty());
        assert!(form.amount.is_empty());
        assert!(form.notes.is_empty());
    }

    #[test]
    fn form_parse_amount_valid() {
        let form = SubscriptionForm { amount: "9.99".into(), ..Default::default() };
        assert_eq!(form.parse_amount(), Some(Decimal::new(999, 2)));
    }

    #[test]
    fn form_parse_amount_invalid() {
        let form = SubscriptionForm { amount: "abc".into(), ..Default::default() };
        assert!(form.parse_amount().is_none());
    }

    #[test]
    fn form_parse_start_date_valid() {
        let form = SubscriptionForm { start_date: "2024-06-15".into(), ..Default::default() };
        assert!(form.parse_start_date().is_some());
    }

    #[test]
    fn form_parse_start_date_invalid() {
        let form = SubscriptionForm { start_date: "not-a-date".into(), ..Default::default() };
        assert!(form.parse_start_date().is_none());
    }

    #[test]
    fn form_parse_category_id_valid() {
        let uuid = Uuid::new_v4();
        let form = SubscriptionForm { category_id: uuid.to_string(), ..Default::default() };
        assert_eq!(form.parse_category_id(), Some(uuid));
    }

    #[test]
    fn form_parse_category_id_empty() {
        let form = SubscriptionForm::default();
        assert!(form.parse_category_id().is_none());
    }

    #[test]
    fn form_parse_category_id_invalid() {
        let form = SubscriptionForm { category_id: "not-a-uuid".into(), ..Default::default() };
        assert!(form.parse_category_id().is_none());
    }

    // ── ValidationResult tests ──

    #[test]
    fn validation_result_default_is_valid() {
        assert!(ValidationResult::default().is_valid());
    }

    #[test]
    fn validation_result_with_error_is_invalid() {
        let result = ValidationResult {
            name_error: Some("Required".into()),
            ..Default::default()
        };
        assert!(!result.is_valid());
    }

    // ── validate_subscription_form tests ──

    #[test]
    fn validate_empty_name_fails() {
        let result = validate_subscription_form("", "10", "EUR", "monthly", "2024-01-01", "cat");
        assert!(result.name_error.is_some());
    }

    #[test]
    fn validate_short_name_fails() {
        let result = validate_subscription_form("A", "10", "EUR", "monthly", "2024-01-01", "cat");
        assert!(result.name_error.is_some());
        assert_eq!(result.name_error.unwrap(), "Name must be at least 2 characters");
    }

    #[test]
    fn validate_empty_amount_fails() {
        let result = validate_subscription_form("Netflix", "", "EUR", "monthly", "2024-01-01", "cat");
        assert!(result.amount_error.is_some());
        assert_eq!(result.amount_error.unwrap(), "Amount is required");
    }

    #[test]
    fn validate_negative_amount_fails() {
        let result = validate_subscription_form("Netflix", "-5", "EUR", "monthly", "2024-01-01", "cat");
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validate_zero_amount_fails() {
        let result = validate_subscription_form("Netflix", "0", "EUR", "monthly", "2024-01-01", "cat");
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validate_invalid_amount_format() {
        let result = validate_subscription_form("Netflix", "abc", "EUR", "monthly", "2024-01-01", "cat");
        assert!(result.amount_error.is_some());
        assert_eq!(result.amount_error.unwrap(), "Invalid amount format");
    }

    #[test]
    fn validate_empty_currency_fails() {
        let result = validate_subscription_form("Netflix", "10", "", "monthly", "2024-01-01", "cat");
        assert!(result.currency_error.is_some());
    }

    #[test]
    fn validate_empty_frequency_fails() {
        let result = validate_subscription_form("Netflix", "10", "EUR", "", "2024-01-01", "cat");
        assert!(result.frequency_error.is_some());
    }

    #[test]
    fn validate_empty_date_fails() {
        let result = validate_subscription_form("Netflix", "10", "EUR", "monthly", "", "cat");
        assert!(result.start_date_error.is_some());
        assert_eq!(result.start_date_error.unwrap(), "Start date is required");
    }

    #[test]
    fn validate_invalid_date_format() {
        let result = validate_subscription_form("Netflix", "10", "EUR", "monthly", "not-a-date", "cat");
        assert!(result.start_date_error.is_some());
        assert_eq!(result.start_date_error.unwrap(), "Invalid date format");
    }

    #[test]
    fn validate_empty_category_fails() {
        let result = validate_subscription_form("Netflix", "10", "EUR", "monthly", "2024-01-01", "");
        assert!(result.category_error.is_some());
    }

    #[test]
    fn validate_valid_form_succeeds() {
        let result = validate_subscription_form("Netflix", "9.99", "EUR", "monthly", "2024-01-01", "cat-1");
        assert!(result.is_valid());
        assert!(result.name_error.is_none());
        assert!(result.amount_error.is_none());
        assert!(result.currency_error.is_none());
        assert!(result.frequency_error.is_none());
        assert!(result.start_date_error.is_none());
        assert!(result.category_error.is_none());
    }
}