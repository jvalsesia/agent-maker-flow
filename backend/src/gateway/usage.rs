//! Redis usage/cost counters (backend accounting; not surfaced in the UI).
//!
//! Per-model and global hashes accumulate token, request, and cost totals.
//! Token/request fields use `HINCRBY`; cost uses `HINCRBYFLOAT`. All increments
//! are best-effort — counter failures never fail the originating request.

use deadpool_redis::redis;
use deadpool_redis::Connection;

use crate::gateway::GatewayClient;

async fn hincrby(conn: &mut Connection, key: &str, field: &str, by: i64) {
    let _: redis::RedisResult<i64> = redis::cmd("HINCRBY")
        .arg(key)
        .arg(field)
        .arg(by)
        .query_async(conn)
        .await;
}

async fn hincrbyfloat(conn: &mut Connection, key: &str, field: &str, by: f64) {
    let _: redis::RedisResult<f64> = redis::cmd("HINCRBYFLOAT")
        .arg(key)
        .arg(field)
        .arg(by)
        .query_async(conn)
        .await;
}

impl GatewayClient {
    /// Increment per-model and global token/request/cost counters for one
    /// billed request.
    pub async fn record_usage(
        &self,
        model: &str,
        prompt_tokens: u64,
        completion_tokens: u64,
        cost_usd: f64,
    ) {
        let Ok(mut conn) = self.redis().get().await else {
            return;
        };

        let model_key = format!("usage:model:{model}");
        for key in [model_key.as_str(), "usage:global"] {
            hincrby(&mut conn, key, "prompt_tokens", prompt_tokens as i64).await;
            hincrby(&mut conn, key, "completion_tokens", completion_tokens as i64).await;
            hincrby(&mut conn, key, "requests", 1).await;
            hincrbyfloat(&mut conn, key, "cost_usd", cost_usd).await;
        }
    }

    /// Increment the global cache-hit and request counters (a cache hit is a
    /// served request but is not re-billed).
    pub async fn record_cache_hit(&self) {
        let Ok(mut conn) = self.redis().get().await else {
            return;
        };
        hincrby(&mut conn, "usage:global", "cache_hits", 1).await;
        hincrby(&mut conn, "usage:global", "requests", 1).await;
    }
}
