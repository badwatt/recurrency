use crate::models::account::{Account, CreateAccountRequest, UpdateAccountRequest};
use crate::models::category::{Category, CreateCategoryRequest, UpdateCategoryRequest};
use crate::models::subscription::{CalendarItem, CreateSubscriptionRequest, Subscription, UpdateSubscriptionRequest};
use crate::models::transaction::{CreateTransactionRequest, Transaction, UpdateTransactionRequest};
use crate::api::client::ApiError;

/// Abstract storage interface.
/// In Server mode: delegates to API calls.
/// In Offline mode: delegates to local JSON/file storage.
pub struct Storage;

impl Storage {
    // ── Subscriptions ──

    pub async fn list_subscriptions() -> Result<Vec<Subscription>, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::list_subscriptions().await
        } else {
            crate::api::subscriptions::list().await
        }
    }

    pub async fn create_subscription(req: &CreateSubscriptionRequest) -> Result<Subscription, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::create_subscription(req).await
        } else {
            crate::api::subscriptions::create(req).await
        }
    }

    pub async fn update_subscription(id: &uuid::Uuid, req: &UpdateSubscriptionRequest) -> Result<Subscription, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::update_subscription(id, req).await
        } else {
            crate::api::subscriptions::update(id, req).await
        }
    }

    pub async fn delete_subscription(id: &uuid::Uuid) -> Result<(), ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::delete_subscription(id).await
        } else {
            crate::api::subscriptions::delete(id).await
        }
    }

    pub async fn calendar_month(year: i32, month: u32) -> Result<Vec<CalendarItem>, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::calendar_month(year, month).await
        } else {
            crate::api::subscriptions::calendar_month(year, month).await
        }
    }

    // ── Transactions ──

    pub async fn list_transactions() -> Result<Vec<Transaction>, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::list_transactions().await
        } else {
            crate::api::transactions::list().await
        }
    }

    pub async fn create_transaction(req: &CreateTransactionRequest) -> Result<Transaction, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::create_transaction(req).await
        } else {
            crate::api::transactions::create(req).await
        }
    }

    pub async fn update_transaction(id: &uuid::Uuid, req: &UpdateTransactionRequest) -> Result<Transaction, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::update_transaction(id, req).await
        } else {
            crate::api::transactions::update(id, req).await
        }
    }

    pub async fn delete_transaction(id: &uuid::Uuid) -> Result<(), ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::delete_transaction(id).await
        } else {
            crate::api::transactions::delete(id).await
        }
    }

    // ── Accounts ──

    pub async fn list_accounts() -> Result<Vec<Account>, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::list_accounts().await
        } else {
            crate::api::accounts::list().await
        }
    }

    pub async fn create_account(req: &CreateAccountRequest) -> Result<Account, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::create_account(req).await
        } else {
            crate::api::accounts::create(req).await
        }
    }

    pub async fn update_account(id: &uuid::Uuid, req: &UpdateAccountRequest) -> Result<Account, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::update_account(id, req).await
        } else {
            crate::api::accounts::update(id, req).await
        }
    }

    pub async fn delete_account(id: &uuid::Uuid) -> Result<(), ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::delete_account(id).await
        } else {
            crate::api::accounts::delete(id).await
        }
    }

    pub async fn set_default_account(id: &uuid::Uuid) -> Result<(), ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::set_default_account(id).await
        } else {
            crate::api::accounts::set_default(id).await
        }
    }

    // ── Categories ──

    pub async fn list_categories() -> Result<Vec<Category>, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::list_categories().await
        } else {
            crate::api::categories::list().await
        }
    }

    pub async fn create_category(req: &CreateCategoryRequest) -> Result<Category, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::create_category(req).await
        } else {
            crate::api::categories::create(req).await
        }
    }

    pub async fn update_category(id: &uuid::Uuid, req: &UpdateCategoryRequest) -> Result<Category, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::update_category(id, req).await
        } else {
            crate::api::categories::update(id, req).await
        }
    }

    pub async fn delete_category(id: &uuid::Uuid) -> Result<(), ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::delete_category(id).await
        } else {
            crate::api::categories::delete(id).await
        }
    }

    pub async fn set_default_category(id: &uuid::Uuid) -> Result<(), ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::set_default_category(id).await
        } else {
            crate::api::categories::set_default(id).await
        }
    }

    // ── Reports ──

    pub async fn get_report_summary(year: i32, month: u32) -> Result<crate::models::report::ReportResponse, ApiError> {
        if crate::api::client::is_offline_mode() {
            crate::storage::local::get_report_summary(year, month).await
        } else {
            crate::api::reports::get_summary(year, month).await
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn storage_module_compiles() {
        // Storage is a stateless dispatcher, no constructor needed
        assert!(true);
    }
}