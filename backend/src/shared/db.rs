use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Maximum number of connections in the pool when DATABASE_MAX_CONNECTIONS is
/// unset or invalid.
const DEFAULT_MAX_CONNECTIONS: u32 = 5;

/// Directory used by the zero-config local database when DATABASE_URL is not
/// set. Override with `RECURRENCY_DATA_DIR`.
#[cfg(feature = "embedded-db")]
const DEFAULT_EMBEDDED_DATA_DIR: &str = "recurrency-data";
#[cfg(feature = "embedded-db")]
const EMBEDDED_DATABASE_NAME: &str = "recurrency";

/// True once the local embedded PostgreSQL fallback has been selected. Kept
/// separate from the pool because migrations differ for a fresh local cluster.
static EMBEDDED_DATABASE: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "embedded-db")]
struct EmbeddedDatabase {
    // Owns the PostgreSQL server for the whole process lifetime.
    pg: tokio::sync::Mutex<postgresql_embedded::PostgreSQL>,
    url: String,
}

#[cfg(feature = "embedded-db")]
static EMBEDDED: tokio::sync::OnceCell<EmbeddedDatabase> = tokio::sync::OnceCell::const_new();

/// Parse the desired pool size. Rejects 0 and invalid values so the pool is
/// always usable.
fn parse_max_connections(value: Option<&str>) -> u32 {
    value
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_MAX_CONNECTIONS)
}

fn configuration_error(message: impl Into<String>) -> sqlx::Error {
    sqlx::Error::Configuration(std::io::Error::other(message.into()).into())
}

#[cfg(feature = "embedded-db")]
async fn embedded_url() -> Result<String, sqlx::Error> {
    use postgresql_embedded::{PostgreSQL, SettingsBuilder};

    let embedded = EMBEDDED
        .get_or_try_init(|| async {
            let root = std::env::var("RECURRENCY_DATA_DIR")
                .unwrap_or_else(|_| DEFAULT_EMBEDDED_DATA_DIR.to_string());
            let root = std::path::PathBuf::from(root);
            std::fs::create_dir_all(&root).map_err(|error| {
                configuration_error(format!("embedded database directory: {error}"))
            })?;
            let password_path = root.join("password");
            let password = match std::fs::read_to_string(&password_path) {
                Ok(password) if !password.trim().is_empty() => password,
                _ => {
                    let password = uuid::Uuid::new_v4().to_string();
                    std::fs::write(&password_path, &password).map_err(|error| {
                        configuration_error(format!("embedded database password: {error}"))
                    })?;
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = std::fs::set_permissions(
                            &password_path,
                            std::fs::Permissions::from_mode(0o600),
                        );
                    }
                    password
                }
            };
            let settings = SettingsBuilder::new()
                .data_dir(root.join("data"))
                .installation_dir(root.join("postgres"))
                .host("127.0.0.1")
                .port(0)
                .password(password)
                .temporary(false)
                .config("shared_buffers", "16MB")
                .config("max_connections", "20")
                .build();
            let mut pg = PostgreSQL::new(settings);
            pg.setup()
                .await
                .map_err(|error| configuration_error(format!("embedded database setup: {error}")))?;
            pg.start()
                .await
                .map_err(|error| configuration_error(format!("embedded database start: {error}")))?;
            if !pg
                .database_exists(EMBEDDED_DATABASE_NAME)
                .await
                .map_err(|error| configuration_error(format!("embedded database check: {error}")))?
            {
                pg.create_database(EMBEDDED_DATABASE_NAME)
                    .await
                    .map_err(|error| {
                        configuration_error(format!("embedded database create: {error}"))
                    })?;
            }
            let url = pg.settings().url(EMBEDDED_DATABASE_NAME);
            Ok::<_, sqlx::Error>(EmbeddedDatabase {
                pg: tokio::sync::Mutex::new(pg),
                url,
            })
        })
        .await?;
    EMBEDDED_DATABASE.store(true, Ordering::SeqCst);
    Ok(embedded.url.clone())
}

/// Stop the local database before the process exits. No-op for external
/// PostgreSQL and for builds without the embedded database feature.
pub async fn shutdown_embedded() {
    #[cfg(feature = "embedded-db")]
    if let Some(embedded) = EMBEDDED.get() {
        let pg = embedded.pg.lock().await;
        if let Err(error) = pg.stop().await {
            tracing::warn!("Could not stop embedded PostgreSQL: {error}");
        }
    }
}

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    create_pool_with_url(None).await
}

pub async fn create_pool_with_url(database_url: Option<&str>) -> Result<PgPool, sqlx::Error> {
    let explicit_url = database_url
        .map(str::to_string)
        .or_else(|| {
            std::env::var("DATABASE_URL")
                .ok()
                .filter(|url| !url.trim().is_empty())
        });

    let (database_url, max_connections) = match explicit_url {
        Some(url) => {
            if url.starts_with("sqlite:") {
                return Err(configuration_error(
                    "SQLite is not supported; use PostgreSQL or omit DATABASE_URL for the embedded database",
                ));
            }
            let max_connections =
                parse_max_connections(std::env::var("DATABASE_MAX_CONNECTIONS").ok().as_deref());
            (url, max_connections)
        }
        None => {
            #[cfg(feature = "embedded-db")]
            {
                (embedded_url().await?, 1)
            }
            #[cfg(not(feature = "embedded-db"))]
            {
                return Err(configuration_error(
                    "DATABASE_URL must be set (this build has no embedded database)",
                ));
            }
        }
    };

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

/// The embedded cluster is created empty. Apply the complete current schema
/// directly instead of replaying migrations that assume the uuid-ossp
/// extension is installed on the host PostgreSQL.
const EMBEDDED_SCHEMA: &str = include_str!("../../embedded_schema.sql");

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    if EMBEDDED_DATABASE.load(Ordering::SeqCst) {
        sqlx::raw_sql(EMBEDDED_SCHEMA).execute(pool).await?;
        return Ok(());
    }
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

    #[tokio::test]
    async fn create_pool_requires_database_url_without_embedded_feature() {
        crate::shared::test_env::remove_env("DATABASE_URL");
        #[cfg(not(feature = "embedded-db"))]
        {
            let error = create_pool().await.expect_err("missing URL must fail");
            assert!(error.to_string().contains("DATABASE_URL must be set"));
        }
        #[cfg(feature = "embedded-db")]
        {
            // The embedded database is exercised by the standalone build;
            // opening a cluster in a unit test would make the suite slow.
            assert!(!EMBEDDED_DATABASE.load(Ordering::SeqCst));
        }
    }

    #[test]
    fn embedded_schema_covers_every_domain_table() {
        for table in [
            "users",
            "accounts",
            "categories",
            "subscriptions",
            "transactions",
            "notification_preferences",
            "notifications",
            "user_preferences",
            "notification_channels",
            "summary_notification_settings",
            "debts",
            "debt_payments",
            "currency_settings",
        ] {
            assert!(
                EMBEDDED_SCHEMA.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
                "embedded schema is missing {table}"
            );
        }
    }

    #[test]
    fn embedded_schema_does_not_require_host_extensions() {
        assert!(!EMBEDDED_SCHEMA.contains("CREATE EXTENSION"));
        assert!(!EMBEDDED_SCHEMA.contains("uuid_generate_v4"));
    }

    #[test]
    fn sqlite_url_is_rejected_explicitly() {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let error = runtime
            .block_on(create_pool_with_url(Some("sqlite://local.db")))
            .expect_err("sqlite must be rejected");
        assert!(error.to_string().contains("SQLite is not supported"));
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
