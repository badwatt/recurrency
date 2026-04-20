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
