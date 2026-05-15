//! Re-exports from entry_form.rs for backward compatibility.
//!
//! New code should use `components::entry_form` directly.

pub use crate::components::entry_form::{
    EntryForm as SubscriptionForm,
    ValidationResult,
    validate_subscription_form,
    validate_entry_form,
    DialogForm,
    EditForm,
    EntityType,
    DialogMode,
    EntryForm,
    TransactionValidationResult,
    validate_transaction_form,
};