//! Shared, cloneable application state injected into handlers.

use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;

use crate::auth::AuthState;
use crate::config::AppConfig;

/// Carries the database pool, cache pool, configuration, and authentication
/// state. Cheap to clone (pools, config, and the `Arc`-wrapped auth state are
/// reference-counted / cloneable handles).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub config: AppConfig,
    pub auth: Arc<AuthState>,
}
