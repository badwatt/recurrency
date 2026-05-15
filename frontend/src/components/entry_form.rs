//! Shared entry form for both subscriptions and transactions.
//!
//! Single form handles add/edit for both entity types.
//! `EntityType` prop controls which fields are shown/required.

use rust_decimal::Decimal;
use uuid::Uuid;

/// Which entity the form represents.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityType {
    Subscription,
    Transaction,
}

/// Which mode the dialog is in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DialogMode {
    Add,
    Edit,
}

/// Shared form state for subscription/transaction create/edit dialogs.
#[derive(Debug, Clone, Default)]
pub struct EntryForm {
    pub name: String,
    pub amount: String,
    pub currency: String,
    /// Subscription-only: "weekly", "monthly", "quarterly", "yearly".
    /// Empty for transactions.
    pub frequency: String,
    /// Date field: maps to `start_date` (subscription) or `date` (transaction).
    pub start_date: String,
    pub category_id: String,
    pub notes: String,
    pub logo_url: String,
}

impl EntryForm {
    /// Reset form to default values.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Initialize form for creating a new entry.
    pub fn init_for_create(date: &str, currency: &str, category_id: &str, entity: EntityType) -> Self {
        Self {
            name: String::new(),
            amount: String::new(),
            currency: currency.to_string(),
            frequency: match entity {
                EntityType::Subscription => "monthly".to_string(),
                EntityType::Transaction => String::new(),
            },
            start_date: date.to_string(),
            category_id: category_id.to_string(),
            notes: String::new(),
            logo_url: String::new(),
        }
    }

    /// Initialize form from existing entry data (for editing).
    pub fn from_entry(
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

    /// Parse the category_id field as Uuid, returning None if invalid or empty.
    pub fn parse_category_id(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.category_id).ok()
    }

    /// Build an UpdateSubscriptionRequest from the form fields.
    /// Returns None if amount cannot be parsed.
    pub fn to_subscription_update_request(
        &self,
    ) -> Option<crate::models::subscription::UpdateSubscriptionRequest> {
        let amount = self.parse_amount()?;
        let category = self.parse_category_id();
        Some(crate::models::subscription::UpdateSubscriptionRequest {
            name: Some(self.name.clone()),
            amount: Some(amount),
            currency: Some(self.currency.clone()),
            frequency: Some(self.frequency.clone()),
            start_date: Some(self.parse_start_date().unwrap_or(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date"),
            )),
            category_id: category,
            notes: if self.notes.is_empty() { None } else { Some(self.notes.clone()) },
            logo_url: if self.logo_url.is_empty() { None } else { Some(self.logo_url.clone()) },
            ..Default::default()
        })
    }

    /// Build a CreateSubscriptionRequest from the form fields.
    /// Returns None if amount or date cannot be parsed.
    pub fn to_subscription_create_request(
        &self,
    ) -> Option<crate::models::subscription::CreateSubscriptionRequest> {
        let amount = self.parse_amount()?;
        let category = self.parse_category_id();
        Some(crate::models::subscription::CreateSubscriptionRequest {
            name: self.name.clone(),
            amount,
            currency: self.currency.clone(),
            frequency: self.frequency.clone(),
            start_date: self.parse_start_date().unwrap_or(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date"),
            ),
            category_id: category,
            notes: if self.notes.is_empty() { None } else { Some(self.notes.clone()) },
            logo_url: if self.logo_url.is_empty() { None } else { Some(self.logo_url.clone()) },
        })
    }

    /// Build an UpdateTransactionRequest from the form fields.
    /// Returns None if amount cannot be parsed.
    pub fn to_transaction_update_request(
        &self,
    ) -> Option<crate::models::transaction::UpdateTransactionRequest> {
        let amount = self.parse_amount()?;
        let category = self.parse_category_id();
        Some(crate::models::transaction::UpdateTransactionRequest {
            name: Some(self.name.clone()),
            amount: Some(amount),
            currency: Some(self.currency.clone()),
            date: Some(self.parse_start_date().unwrap_or(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date"),
            )),
            category_id: category,
            notes: if self.notes.is_empty() { None } else { Some(self.notes.clone()) },
            logo_url: if self.logo_url.is_empty() { None } else { Some(self.logo_url.clone()) },
            ..Default::default()
        })
    }

    /// Build a CreateTransactionRequest from the form fields.
    /// Returns None if amount or date cannot be parsed.
    pub fn to_transaction_create_request(
        &self,
    ) -> Option<crate::models::transaction::CreateTransactionRequest> {
        let amount = self.parse_amount()?;
        let category = self.parse_category_id();
        Some(crate::models::transaction::CreateTransactionRequest {
            name: self.name.clone(),
            amount,
            currency: self.currency.clone(),
            date: self.parse_start_date().unwrap_or(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date"),
            ),
            category_id: category,
            notes: if self.notes.is_empty() { None } else { Some(self.notes.clone()) },
            logo_url: if self.logo_url.is_empty() { None } else { Some(self.logo_url.clone()) },
        })
    }

    /// Human-readable label for the date field.
    pub fn date_label(&self, entity: EntityType) -> &'static str {
        match entity {
            EntityType::Subscription => "Start date",
            EntityType::Transaction => "Date",
        }
    }

    /// Whether the frequency field is relevant for this entity type.
    pub fn has_frequency(&self, entity: EntityType) -> bool {
        matches!(entity, EntityType::Subscription)
    }
}

/// Validation result for entry forms.
/// `frequency_error` only applies to subscriptions.
/// `start_date_error` is used for both (start_date for subscriptions, date for transactions).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ValidationResult {
    pub name_error: Option<String>,
    pub amount_error: Option<String>,
    pub currency_error: Option<String>,
    /// Only set for EntityType::Subscription.
    pub frequency_error: Option<String>,
    /// Used for start_date (subscription) or date (transaction).
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

/// Pure validation function for entry forms.
/// `entity` controls whether frequency is required.
pub fn validate_entry_form(
    name: &str,
    amount: &str,
    currency: &str,
    frequency: &str,
    start_date: &str,
    category_id: &str,
    entity: EntityType,
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

    // Frequency only required for subscriptions
    if matches!(entity, EntityType::Subscription) && frequency.is_empty() {
        result.frequency_error = Some("Frequency is required".into());
    }

    if start_date.is_empty() {
        let label = match entity {
            EntityType::Subscription => "Start date",
            EntityType::Transaction => "Date",
        };
        result.start_date_error = Some(format!("{} is required", label));
    } else if start_date.parse::<chrono::NaiveDate>().is_err() {
        result.start_date_error = Some("Invalid date format".into());
    }

    if category_id.is_empty() {
        result.category_error = Some("Category is required".into());
    }

    result
}

/// Backward-compatible alias — validates a subscription form.
pub fn validate_subscription_form(
    name: &str,
    amount: &str,
    currency: &str,
    frequency: &str,
    start_date: &str,
    category_id: &str,
) -> ValidationResult {
    validate_entry_form(name, amount, currency, frequency, start_date, category_id, EntityType::Subscription)
}

/// Transaction-specific validation result (no frequency_error).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TransactionValidationResult {
    pub name_error: Option<String>,
    pub amount_error: Option<String>,
    pub currency_error: Option<String>,
    pub date_error: Option<String>,
    pub category_error: Option<String>,
}

impl TransactionValidationResult {
    pub fn is_valid(&self) -> bool {
        self.name_error.is_none()
            && self.amount_error.is_none()
            && self.currency_error.is_none()
            && self.date_error.is_none()
            && self.category_error.is_none()
    }
}

/// Backward-compatible alias — validates a transaction form.
pub fn validate_transaction_form(
    name: &str,
    amount: &str,
    currency: &str,
    date: &str,
    category_id: &str,
) -> TransactionValidationResult {
    let result = validate_entry_form(name, amount, currency, "", date, category_id, EntityType::Transaction);
    TransactionValidationResult {
        name_error: result.name_error,
        amount_error: result.amount_error,
        currency_error: result.currency_error,
        date_error: result.start_date_error,
        category_error: result.category_error,
    }
}

// Backward-compatible type aliases
pub type SubscriptionForm = EntryForm;
pub type EditForm = EntryForm;
pub type DialogForm = EntryForm;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_default_has_empty_fields() {
        let form = EntryForm::default();
        assert!(form.name.is_empty());
        assert!(form.amount.is_empty());
        assert!(form.frequency.is_empty());
        assert!(form.start_date.is_empty());
        assert!(form.category_id.is_empty());
    }

    #[test]
    fn form_init_for_create_subscription() {
        let form = EntryForm::init_for_create("2024-06-15", "USD", "cat-123", EntityType::Subscription);
        assert!(form.name.is_empty());
        assert_eq!(form.frequency, "monthly");
        assert_eq!(form.start_date, "2024-06-15");
    }

    #[test]
    fn form_init_for_create_transaction() {
        let form = EntryForm::init_for_create("2024-06-15", "USD", "cat-123", EntityType::Transaction);
        assert!(form.frequency.is_empty());
        assert_eq!(form.start_date, "2024-06-15");
    }

    #[test]
    fn form_from_entry_populates_all_fields() {
        let form = EntryForm::from_entry(
            "Netflix", "9.99", "EUR", "monthly", "2024-01-01", "cat-456", "notes", "logo.png",
        );
        assert_eq!(form.name, "Netflix");
        assert_eq!(form.frequency, "monthly");
        assert_eq!(form.start_date, "2024-01-01");
    }

    #[test]
    fn form_reset_clears_all_fields() {
        let mut form = EntryForm::from_entry(
            "Netflix", "9.99", "EUR", "monthly", "2024-01-01", "cat-456", "notes", "logo.png",
        );
        form.reset();
        assert!(form.name.is_empty());
    }

    #[test]
    fn form_parse_amount_valid() {
        let form = EntryForm { amount: "9.99".into(), ..Default::default() };
        assert_eq!(form.parse_amount(), Some(Decimal::new(999, 2)));
    }

    #[test]
    fn form_parse_amount_invalid() {
        let form = EntryForm { amount: "abc".into(), ..Default::default() };
        assert!(form.parse_amount().is_none());
    }

    #[test]
    fn form_parse_start_date_valid() {
        let form = EntryForm { start_date: "2024-06-15".into(), ..Default::default() };
        assert!(form.parse_start_date().is_some());
    }

    #[test]
    fn form_date_label_subscription() {
        let form = EntryForm::default();
        assert_eq!(form.date_label(EntityType::Subscription), "Start date");
    }

    #[test]
    fn form_date_label_transaction() {
        let form = EntryForm::default();
        assert_eq!(form.date_label(EntityType::Transaction), "Date");
    }

    #[test]
    fn form_has_frequency_subscription() {
        let form = EntryForm::default();
        assert!(form.has_frequency(EntityType::Subscription));
    }

    #[test]
    fn form_has_frequency_transaction() {
        let form = EntryForm::default();
        assert!(!form.has_frequency(EntityType::Transaction));
    }

    #[test]
    fn validation_result_default_is_valid() {
        assert!(ValidationResult::default().is_valid());
    }

    #[test]
    fn validation_result_with_error_is_invalid() {
        let result = ValidationResult { name_error: Some("Required".into()), ..Default::default() };
        assert!(!result.is_valid());
    }

    #[test]
    fn validate_subscription_empty_name_fails() {
        let result = validate_entry_form("", "10", "EUR", "monthly", "2024-01-01", "cat", EntityType::Subscription);
        assert!(result.name_error.is_some());
    }

    #[test]
    fn validate_transaction_empty_name_fails() {
        let result = validate_entry_form("", "10", "EUR", "", "2024-01-01", "cat", EntityType::Transaction);
        assert!(result.name_error.is_some());
    }

    #[test]
    fn validate_subscription_empty_frequency_fails() {
        let result = validate_entry_form("Netflix", "10", "EUR", "", "2024-01-01", "cat", EntityType::Subscription);
        assert!(result.frequency_error.is_some());
    }

    #[test]
    fn validate_transaction_empty_frequency_ok() {
        let result = validate_entry_form("Coffee", "10", "EUR", "", "2024-01-01", "cat", EntityType::Transaction);
        assert!(result.frequency_error.is_none());
    }

    #[test]
    fn validate_empty_date_subscription_label() {
        let result = validate_entry_form("Netflix", "10", "EUR", "monthly", "", "cat", EntityType::Subscription);
        assert_eq!(result.start_date_error.unwrap(), "Start date is required");
    }

    #[test]
    fn validate_empty_date_transaction_label() {
        let result = validate_entry_form("Coffee", "10", "EUR", "", "", "cat", EntityType::Transaction);
        assert_eq!(result.start_date_error.unwrap(), "Date is required");
    }

    #[test]
    fn validate_valid_subscription_succeeds() {
        let result = validate_entry_form("Netflix", "9.99", "EUR", "monthly", "2024-01-01", "cat-1", EntityType::Subscription);
        assert!(result.is_valid());
    }

    #[test]
    fn validate_valid_transaction_succeeds() {
        let result = validate_entry_form("Coffee", "4.50", "USD", "", "2024-01-01", "cat-1", EntityType::Transaction);
        assert!(result.is_valid());
        assert!(result.frequency_error.is_none());
    }

    #[test]
    fn backward_compat_validate_subscription() {
        let result = validate_subscription_form("Netflix", "10", "EUR", "monthly", "2024-01-01", "cat");
        assert!(result.is_valid());
    }

    #[test]
    fn backward_compat_validate_transaction() {
        let result = validate_transaction_form("Coffee", "4.50", "USD", "2024-01-01", "cat");
        assert!(result.is_valid());
        assert!(result.name_error.is_none());
        assert!(result.date_error.is_none());
    }

    #[test]
    fn transaction_form_empty_date_fails() {
        let result = validate_transaction_form("Coffee", "4.50", "USD", "", "cat");
        assert!(result.date_error.is_some());
    }

    #[test]
    fn transaction_form_invalid_amount() {
        let result = validate_transaction_form("Coffee", "abc", "USD", "2024-01-01", "cat");
        assert!(result.amount_error.is_some());
    }
}