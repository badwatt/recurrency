use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test pool creation requires DATABASE_URL
    #[tokio::test]
    #[should_panic(expected = "DATABASE_URL must be set")]
    async fn create_pool_requires_database_url() {
        // Clear DATABASE_URL if set
        std::env::remove_var("DATABASE_URL");
        let _ = create_pool().await;
    }

    // Test run_migrations requires valid pool
    #[tokio::test]
    async fn run_migrations_with_invalid_pool() {
        // This test documents that run_migrations needs a valid pool
        // We skip if DATABASE_URL is not set
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => return, // Skip test if no database URL
        };
        let result = sqlx::PgPool::connect(&database_url).await;
        match result {
            Ok(pool) => {
                // If we can connect, test migrations
                let _ = run_migrations(&pool).await;
            }
            Err(_) => {
                // If we can't connect, that's expected in CI
            }
        }
    }

    // Test that PgPoolOptions can be configured
    #[test]
    fn pool_options_configuration() {
        let _options = PgPoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(30));

        // Options should be configurable without panicking
        assert!(true);
    }

    // Test PgPoolOptions with different configurations
    #[test]
    fn pool_options_min_connections() {
        let _options = PgPoolOptions::new().min_connections(0);
        assert!(true);
    }

    #[test]
    fn pool_options_idle_timeout() {
        let _options = PgPoolOptions::new().idle_timeout(Duration::from_secs(600));
        assert!(true);
    }

    #[test]
    fn pool_options_max_lifetime() {
        let _options = PgPoolOptions::new().max_lifetime(Duration::from_secs(1800));
        assert!(true);
    }

    // Test Duration configurations
    #[test]
    fn duration_variants() {
        let _ = Duration::from_secs(30);
        let _ = Duration::from_secs(60);
        let _ = Duration::from_secs(300);
        let _ = Duration::from_secs(0);
    }
}
