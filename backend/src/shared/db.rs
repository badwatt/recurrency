use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool() -> anyhow::Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://recur:recur_dev_password@localhost:5432/recur".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    tracing::info!("Running database migrations...");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    tracing::info!("Migrations complete");
    Ok(())
}
