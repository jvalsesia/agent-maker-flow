//! Shared, cloneable application state injected into handlers.

use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;

use crate::auth::AuthState;
use crate::config::AppConfig;
use crate::gateway::GatewayClient;
use crate::runs::RunRegistry;

/// Carries the database pool, cache pool, configuration, authentication state,
/// the LLM gateway client, and the F09 in-memory run registry. Cheap to
/// clone (pools, config, and the `Arc`-wrapped auth/gateway/runs handles are
/// reference-counted).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub config: AppConfig,
    pub auth: Arc<AuthState>,
    pub gateway: Arc<GatewayClient>,
    pub runs: Arc<RunRegistry>,
}
