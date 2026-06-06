//! Redis pool setup and connectivity verification.

use anyhow::Context;
use deadpool_redis::{Config, Pool, Runtime};

/// Create the Redis connection pool.
pub fn init_pool(redis_url: &str) -> anyhow::Result<Pool> {
    Config::from_url(redis_url)
        .create_pool(Some(Runtime::Tokio1))
        .context("failed to create Redis pool")
}

/// Verify connectivity with a `PING`.
pub async fn verify(pool: &Pool) -> anyhow::Result<()> {
    let mut conn = pool
        .get()
        .await
        .context("failed to acquire Redis connection")?;

    let pong: String = redis::cmd("PING")
        .query_async(&mut conn)
        .await
        .context("redis PING failed")?;

    if pong != "PONG" {
        anyhow::bail!("unexpected redis PING response: {pong}");
    }
    Ok(())
}
