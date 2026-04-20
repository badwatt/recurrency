use std::sync::Arc;
use crate::auth::service::AuthServiceTrait;

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
/// Pre-registers test users so handler tests can access them.
/// Returns None if DATABASE_URL is not set or connection fails.
#[cfg(test)]
pub async fn create_test_app_state() -> Option<AppState> {
    use crate::shared::mocks::MockAuthService;
    use crate::auth::models::CreateUserRequest;
    use chrono::Utc;

    let database_url = std::env::var("DATABASE_URL").ok()?;
    let pool = sqlx::PgPool::connect(&database_url).await.ok()?;
    let mut mock = MockAuthService::success();

    // Pre-register test user (UUID: 00000000-0000-0000-0000-000000000001)
    if let Ok(user_id) = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001") {
        let user = crate::auth::models::User {
            id: user_id,
            email: "test@test.com".to_string(),
            password_hash: "mock_hash".to_string(),
            name: Some("Test User".to_string()),
            currency: "USD".to_string(),
            verified: false,
            verification_code: Some("123456".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        mock.registered_users.lock().unwrap().push(user);
    }

    let mock: Arc<dyn AuthServiceTrait> = Arc::new(mock);
    Some(AppState::new(pool, mock))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running database
    async fn app_state_new() {
        let state = create_test_app_state().await;
        if state.is_none() {
            return; // No DB available, skip
        }
        let state = state.unwrap();
        let _pool: &sqlx::PgPool = std::ops::Deref::deref(&state);
        let _cloned = state.clone();
        assert!(Arc::strong_count(&state.auth_service) >= 1);
    }

    #[tokio::test]
    #[ignore] // Requires running database
    async fn app_state_verify_token_mock() {
        use crate::auth::service::AuthServiceTrait;
        let state = create_test_app_state().await.expect("DATABASE_URL required");
        let result = state.auth_service.verify_token("any_token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires running database
    async fn app_state_verify_mock_token() {
        use crate::auth::service::AuthServiceTrait;
        let state = create_test_app_state().await.expect("DATABASE_URL required");
        let result = state.auth_service.verify_token(
            "mock_token_00000000-0000-0000-0000-000000000001"
        ).await;
        assert!(result.is_ok());
    }
}
