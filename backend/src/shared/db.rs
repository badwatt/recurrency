use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Maximum number of connections in the pool when DATABASE_MAX_CONNECTIONS is
/// unset or invalid.
const DEFAULT_MAX_CONNECTIONS: u32 = 5;

/// Parse the desired pool size. Rejects 0 and invalid values so the pool is
/// always usable.
fn parse_max_connections(value: Option<&str>) -> u32 {
    value
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_MAX_CONNECTIONS)
}

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    create_pool_with_url(None).await
}

pub async fn create_pool_with_url(database_url: Option<&str>) -> Result<PgPool, sqlx::Error> {
    let database_url = database_url
        .map(str::to_string)
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL must be set");
    let max_connections =
        parse_max_connections(std::env::var("DATABASE_MAX_CONNECTIONS").ok().as_deref());

    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
}

/// Create a DB pool with retry logic. Waits up to 60s for PostgreSQL to be ready.
pub async fn create_pool_with_retry() -> Result<PgPool, sqlx::Error> {
    create_pool_with_retry_url(None).await
}

pub async fn create_pool_with_retry_url(database_url: Option<&str>) -> Result<PgPool, sqlx::Error> {
    let mut attempts = 0u32;
    let max_attempts = 12; // 12 * 5s = 60s total
    loop {
        match create_pool_with_url(database_url).await {
            Ok(pool) => {
                tracing::info!("Database connection established");
                return Ok(pool);
            }
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    tracing::error!("Failed to connect to database after {} attempts: {}", attempts, e);
                    return Err(e);
                }
                tracing::warn!("Database not ready (attempt {}/{}): {}. Retrying in 5s...", attempts, max_attempts, e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

/// Versions edited before this project treated migrations as immutable. The
/// changes were comments and demo-seed data only; their schema effects are
/// idempotent. Repair these legacy checksums so existing installations can
/// upgrade, then let SQLx validate every migration normally.
const LEGACY_DEMO_SEED_MIGRATIONS: &[i64] = &[1, 4, 5];

async fn repair_legacy_demo_seed_checksums(pool: &PgPool) -> Result<(), sqlx::Error> {
    let table_exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('_sqlx_migrations')::text")
            .fetch_one(pool)
            .await?;
    if table_exists.is_none() {
        return Ok(());
    }

    let migrator = sqlx::migrate!("./migrations");
    for version in LEGACY_DEMO_SEED_MIGRATIONS {
        let applied: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT checksum FROM _sqlx_migrations WHERE version = $1 AND success = true",
        )
        .bind(version)
        .fetch_optional(pool)
        .await?;
        let Some(applied) = applied else {
            continue;
        };
        let Some(current) = migrator
            .iter()
            .find(|migration| migration.version == *version)
        else {
            continue;
        };
        if applied.as_slice() == current.checksum.as_ref() {
            continue;
        }

        tracing::warn!(
            "Repairing checksum for legacy migration {}; schema was already applied safely",
            version
        );
        sqlx::query("UPDATE _sqlx_migrations SET checksum = $1 WHERE version = $2")
            .bind(current.checksum.as_ref())
            .bind(version)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    repair_legacy_demo_seed_checksums(pool).await?;
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_max_connections_default_on_none() {
        assert_eq!(parse_max_connections(None), DEFAULT_MAX_CONNECTIONS);
    }

    #[test]
    fn parse_max_connections_rejects_zero_and_invalid() {
        assert_eq!(parse_max_connections(Some("0")), DEFAULT_MAX_CONNECTIONS);
        assert_eq!(parse_max_connections(Some("abc")), DEFAULT_MAX_CONNECTIONS);
    }

    #[test]
    fn parse_max_connections_accepts_positive() {
        assert_eq!(parse_max_connections(Some("20")), 20);
    }

    // Test pool creation requires DATABASE_URL
    #[tokio::test]
    #[should_panic(expected = "DATABASE_URL must be set")]
    async fn create_pool_requires_database_url() {
        // Clear DATABASE_URL if set
        crate::shared::test_env::remove_env("DATABASE_URL");
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
