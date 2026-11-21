use crate::accounts::service::{AccountServiceTrait, SqlxAccountService};
use crate::admin::service::{AdminServiceTrait, SqlxAdminService};
use crate::ai::service::{AiServiceTrait, SqlxAiService};
use crate::auth::service::{AuthServiceTrait, SqlxAuthService};
use crate::backup::service::{BackupServiceTrait, SqlxBackupService};
use crate::categories::service::{CategoryServiceTrait, SqlxCategoryService};
use crate::debts::service::{DebtServiceTrait, SqlxDebtService};
use crate::demo::service::{DemoServiceTrait, SqlxDemoService};
use crate::notifications::service::{NotificationServiceTrait, SqlxNotificationService};
use crate::notifications::settings::{
    NotificationSettingsServiceTrait, SqlxNotificationSettingsService,
};
use crate::preferences::service::{PreferencesServiceTrait, SqlxPreferencesService};
use crate::reports::service::{ReportServiceTrait, SqlxReportService};
use crate::subscriptions::service::{SqlxSubscriptionService, SubscriptionServiceTrait};
use crate::transactions::service::{SqlxTransactionService, TransactionServiceTrait};
use std::sync::Arc;

/// Application state - holds the DB pool and every injectable service port.
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub(crate) auth_service: Arc<dyn AuthServiceTrait>,
    pub(crate) admin_service: Arc<dyn AdminServiceTrait>,
    pub(crate) ai_service: Arc<dyn AiServiceTrait>,
    pub(crate) subscription_service: Arc<dyn SubscriptionServiceTrait>,
    pub(crate) transaction_service: Arc<dyn TransactionServiceTrait>,
    pub(crate) account_service: Arc<dyn AccountServiceTrait>,
    pub(crate) backup_service: Arc<dyn BackupServiceTrait>,
    pub(crate) category_service: Arc<dyn CategoryServiceTrait>,
    pub(crate) demo_service: Arc<dyn DemoServiceTrait>,
    pub(crate) debt_service: Arc<dyn DebtServiceTrait>,
    pub(crate) notification_service: Arc<dyn NotificationServiceTrait>,
    pub(crate) notification_settings_service: Arc<dyn NotificationSettingsServiceTrait>,
    pub(crate) report_service: Arc<dyn ReportServiceTrait>,
    pub(crate) preferences_service: Arc<dyn PreferencesServiceTrait>,
}

impl AppState {
    /// Production wiring: every port backed by a SQLx adapter over the pool.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            auth_service: Arc::new(SqlxAuthService::new(pool.clone())),
            admin_service: Arc::new(SqlxAdminService::new(pool.clone())),
            ai_service: Arc::new(SqlxAiService::new(pool.clone())),
            subscription_service: Arc::new(SqlxSubscriptionService::new(pool.clone())),
            transaction_service: Arc::new(SqlxTransactionService::new(pool.clone())),
            account_service: Arc::new(SqlxAccountService::new(pool.clone())),
            backup_service: Arc::new(SqlxBackupService::new(pool.clone())),
            category_service: Arc::new(SqlxCategoryService::new(pool.clone())),
            demo_service: Arc::new(SqlxDemoService::new(pool.clone())),
            debt_service: Arc::new(SqlxDebtService::new(pool.clone())),
            notification_service: Arc::new(SqlxNotificationService::new(pool.clone())),
            notification_settings_service: Arc::new(SqlxNotificationSettingsService::new(
                pool.clone(),
            )),
            report_service: Arc::new(SqlxReportService::new(pool.clone())),
            preferences_service: Arc::new(SqlxPreferencesService::new(pool.clone())),
            pool,
        }
    }

    /// Test wiring: production adapters plus an injectable auth port.
    #[cfg(test)]
    pub(crate) fn with_auth_service(
        pool: sqlx::PgPool,
        auth_service: Arc<dyn AuthServiceTrait>,
    ) -> Self {
        Self {
            auth_service,
            admin_service: Arc::new(TestAdminService),
            ai_service: Arc::new(TestAiService),
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

#[cfg(test)]
struct TestAdminService;

#[cfg(test)]
#[async_trait::async_trait]
impl AdminServiceTrait for TestAdminService {
    async fn is_admin(&self, _user_id: uuid::Uuid) -> crate::shared::error::AppResult<bool> {
        Ok(false)
    }

    async fn list_users(
        &self,
    ) -> crate::shared::error::AppResult<Vec<crate::admin::models::AdminUser>> {
        Ok(Vec::new())
    }

    async fn settings(
        &self,
    ) -> crate::shared::error::AppResult<crate::admin::models::AdminSettings> {
        Ok(crate::admin::models::AdminSettings {
            registration_enabled: true,
        })
    }

    async fn set_registration_enabled(
        &self,
        enabled: bool,
    ) -> crate::shared::error::AppResult<crate::admin::models::AdminSettings> {
        Ok(crate::admin::models::AdminSettings {
            registration_enabled: enabled,
        })
    }

    async fn full_backup(
        &self,
    ) -> crate::shared::error::AppResult<crate::admin::models::FullBackup> {
        Ok(crate::admin::models::FullBackup {
            format_version: crate::admin::models::FULL_BACKUP_FORMAT_VERSION,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            exported_at: chrono::Utc::now(),
            tables: std::collections::BTreeMap::new(),
        })
    }

    async fn restore_backup(
        &self,
        _backup: crate::admin::models::FullBackup,
    ) -> crate::shared::error::AppResult<crate::admin::models::RestoreResult> {
        Ok(crate::admin::models::RestoreResult { restored_tables: 0 })
    }

    async fn delete_user(
        &self,
        _actor_id: uuid::Uuid,
        _user_id: uuid::Uuid,
    ) -> crate::shared::error::AppResult<()> {
        Ok(())
    }

    async fn set_admin(
        &self,
        _actor_id: uuid::Uuid,
        user_id: uuid::Uuid,
        is_admin: bool,
    ) -> crate::shared::error::AppResult<crate::admin::models::AdminUser> {
        Ok(crate::admin::models::AdminUser {
            id: user_id,
            email: String::new(),
            name: None,
            currency: "EUR".to_string(),
            verified: true,
            is_admin,
            created_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
struct TestAiService;

#[cfg(test)]
#[async_trait::async_trait]
impl AiServiceTrait for TestAiService {
    async fn settings(
        &self,
        _user_id: uuid::Uuid,
    ) -> crate::shared::error::AppResult<crate::ai::models::AiSettingsResponse> {
        Ok(crate::ai::models::AiSettingsResponse {
            enabled: false,
            provider: crate::ai::models::DEFAULT_PROVIDER.to_string(),
            base_url: crate::ai::models::DEFAULT_BASE_URL.to_string(),
            model: crate::ai::models::DEFAULT_MODEL.to_string(),
            configured: false,
        })
    }

    async fn update_settings(
        &self,
        _user_id: uuid::Uuid,
        _req: crate::ai::models::UpdateAiSettingsRequest,
    ) -> crate::shared::error::AppResult<crate::ai::models::AiSettingsResponse> {
        self.settings(_user_id).await
    }

    async fn chat(
        &self,
        _user_id: uuid::Uuid,
        _req: crate::ai::models::AiChatRequest,
    ) -> crate::shared::error::AppResult<crate::ai::models::AiChatResponse> {
        Ok(crate::ai::models::AiChatResponse {
            reply: "AI is not configured in tests.".to_string(),
            actions: Vec::new(),
        })
    }

    async fn models(
        &self,
        _user_id: uuid::Uuid,
        _req: crate::ai::models::AiModelsRequest,
    ) -> crate::shared::error::AppResult<crate::ai::models::AiModelsResponse> {
        Ok(crate::ai::models::AiModelsResponse {
            models: vec![crate::ai::models::DEFAULT_MODEL.to_string()],
        })
    }

    async fn transcribe(
        &self,
        _user_id: uuid::Uuid,
        _req: crate::ai::models::AiTranscriptionRequest,
    ) -> crate::shared::error::AppResult<crate::ai::models::AiTranscriptionResponse> {
        Ok(crate::ai::models::AiTranscriptionResponse {
            text: "test transcription".to_string(),
        })
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
