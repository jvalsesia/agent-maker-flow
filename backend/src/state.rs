//! Shared, cloneable application state injected into handlers.

use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;

use crate::config::AppConfig;

/// Carries the database pool, cache pool, and configuration. Cheap to clone
/// (pools and config are reference-counted / cloneable handles).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub config: AppConfig,
}
