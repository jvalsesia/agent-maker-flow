//! PostgreSQL pool setup, migrations, and connectivity verification.

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Create the PostgreSQL connection pool.
pub async fn init_pool(database_url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("failed to connect to PostgreSQL")
}

/// Apply all embedded migrations from `./migrations`.
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("failed to run database migrations")?;
    Ok(())
}

/// Verify connectivity and that the `pgvector` extension is present.
/// Returns `true` when the `vector` extension is installed.
pub async fn verify(pool: &PgPool) -> anyhow::Result<bool> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .context("database connectivity check failed")?;

    let has_vector: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')")
            .fetch_one(pool)
            .await
            .context("pgvector extension check failed")?;

    Ok(has_vector)
}
