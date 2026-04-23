use crate::auth::service::AuthServiceTrait;
use std::sync::Arc;

/// Application state - holds DB pool and injectable services
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub auth_service: Arc<dyn AuthServiceTrait>,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool, auth_service: Arc<dyn AuthServiceTrait>) -> Self {
        Self { pool, auth_service }
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
    let mock: Arc<dyn AuthServiceTrait> = Arc::new(mock);
    Some(AppState::new(pool, mock))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::service::AuthServiceTrait;
    use crate::shared::mocks::MockAuthService;

    #[tokio::test]
    async fn app_state_new() {
        let mock = MockAuthService::success();
        let mock: Arc<dyn AuthServiceTrait> = Arc::new(mock);
        // connect_lazy requires tokio runtime, works in async context
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let state = AppState::new(pool, mock);
        let _pool: &sqlx::PgPool = std::ops::Deref::deref(&state);
        let _cloned = state.clone();
        assert!(Arc::strong_count(&state.auth_service) >= 1);
    }

    #[tokio::test]
    async fn app_state_verify_token_invalid() {
        let mock = MockAuthService::success();
        let mock: Arc<dyn AuthServiceTrait> = Arc::new(mock);
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let state = AppState::new(pool, mock);
        let result = state.auth_service.verify_token("any_token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn app_state_verify_mock_token_valid() {
        let mock = MockAuthService::success();
        let mock: Arc<dyn AuthServiceTrait> = Arc::new(mock);
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let state = AppState::new(pool, mock);
        let result = state
            .auth_service
            .verify_token("mock_token_00000000-0000-0000-0000-000000000001")
            .await;
        assert!(result.is_ok());
    }
}
