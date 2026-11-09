use crate::accounts::service::{AccountServiceTrait, SqlxAccountService};
use crate::auth::service::{AuthServiceTrait, SqlxAuthService};
use crate::categories::service::{CategoryServiceTrait, SqlxCategoryService};
use crate::demo::service::{DemoServiceTrait, SqlxDemoService};
use crate::notifications::service::{NotificationServiceTrait, SqlxNotificationService};
use crate::preferences::service::{PreferencesServiceTrait, SqlxPreferencesService};
use crate::reports::service::{ReportServiceTrait, SqlxReportService};
use crate::subscriptions::service::{SqlxSubscriptionService, SubscriptionServiceTrait};
use crate::transactions::service::{SqlxTransactionService, TransactionServiceTrait};
use std::sync::Arc;

/// Application state - holds the DB pool and every injectable service port.
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub auth_service: Arc<dyn AuthServiceTrait>,
    pub subscription_service: Arc<dyn SubscriptionServiceTrait>,
    pub transaction_service: Arc<dyn TransactionServiceTrait>,
    pub account_service: Arc<dyn AccountServiceTrait>,
    pub category_service: Arc<dyn CategoryServiceTrait>,
    pub demo_service: Arc<dyn DemoServiceTrait>,
    pub notification_service: Arc<dyn NotificationServiceTrait>,
    pub report_service: Arc<dyn ReportServiceTrait>,
    pub preferences_service: Arc<dyn PreferencesServiceTrait>,
}

impl AppState {
    /// Production wiring: every port backed by a SQLx adapter over the pool.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            auth_service: Arc::new(SqlxAuthService::new(pool.clone())),
            subscription_service: Arc::new(SqlxSubscriptionService::new(pool.clone())),
            transaction_service: Arc::new(SqlxTransactionService::new(pool.clone())),
            account_service: Arc::new(SqlxAccountService::new(pool.clone())),
            category_service: Arc::new(SqlxCategoryService::new(pool.clone())),
            demo_service: Arc::new(SqlxDemoService::new(pool.clone())),
            notification_service: Arc::new(SqlxNotificationService::new(pool.clone())),
            report_service: Arc::new(SqlxReportService::new(pool.clone())),
            preferences_service: Arc::new(SqlxPreferencesService::new(pool.clone())),
            pool,
        }
    }

    /// Test wiring: production adapters plus an injectable auth port.
    #[cfg(test)]
    pub fn with_auth_service(pool: sqlx::PgPool, auth_service: Arc<dyn AuthServiceTrait>) -> Self {
        Self {
            auth_service,
            ..Self::new(pool)
        }
    }
}

impl std::ops::Deref for AppState {
    type Target = sqlx::PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

/// Creates an AppState with mock auth service for testing.
/// Returns None if DATABASE_URL is not set or connection fails.
#[cfg(test)]
pub async fn create_test_app_state() -> Option<AppState> {
    use crate::shared::mocks::MockAuthService;

    let database_url = std::env::var("DATABASE_URL").ok()?;
    let pool = sqlx::PgPool::connect(&database_url).await.ok()?;
    let mock = MockAuthService::success();
    Some(AppState::with_auth_service(pool, Arc::new(mock)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::mocks::MockAuthService;

    #[tokio::test]
    async fn app_state_new() {
        let mock = MockAuthService::success();
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let state = AppState::with_auth_service(pool, Arc::new(mock));
        let _pool: &sqlx::PgPool = std::ops::Deref::deref(&state);
        let _cloned = state.clone();
    }

    #[tokio::test]
    async fn app_state_verify_token_invalid() {
        let mock = MockAuthService::success();
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let state = AppState::with_auth_service(pool, Arc::new(mock));
        let result = state.auth_service.verify_token("any_token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn app_state_verify_mock_token_valid() {
        let mock = MockAuthService::success();
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let state = AppState::with_auth_service(pool, Arc::new(mock));
        let result = state
            .auth_service
            .verify_token("mock_token_00000000-0000-0000-0000-000000000001")
            .await;
        assert!(result.is_ok());
    }
}
