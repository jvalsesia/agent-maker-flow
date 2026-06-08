//! LLM gateway (F03).
//!
//! A single internal client over the LiteLLM proxy: provider/model discovery,
//! chat completions (with exact-match Redis caching), embeddings, and Redis
//! usage/cost counters. Built once at boot and carried in `AppState`.

use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;

use crate::config::GatewayConfig;

pub mod cache;
pub mod catalog;
pub mod completion;
pub mod embedding;
pub mod types;
pub mod usage;

/// Parse the per-request cost from LiteLLM's `x-litellm-response-cost` header;
/// absent/unparseable → `0.0`.
pub(crate) fn parse_cost_header(resp: &reqwest::Response) -> f64 {
    resp.headers()
        .get("x-litellm-response-cost")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or_else(|| {
            tracing::warn!("missing/invalid x-litellm-response-cost header; recording cost 0.0");
            0.0
        })
}

/// Truncate an upstream error body for safe inclusion in a structured error.
pub(crate) fn truncate(mut s: String) -> String {
    const MAX: usize = 200;
    if s.len() > MAX {
        s.truncate(MAX);
        s.push('…');
    }
    s
}

/// The gateway client: a reqwest HTTP client targeting the LiteLLM proxy, plus
/// the Redis pool used for caching and usage counters.
pub struct GatewayClient {
    http: reqwest::Client,
    config: GatewayConfig,
    redis: RedisPool,
}

impl GatewayClient {
    /// Build the client from gateway config and the shared Redis pool.
    pub fn new(config: GatewayConfig, redis: RedisPool) -> Arc<Self> {
        Arc::new(Self {
            http: reqwest::Client::new(),
            config,
            redis,
        })
    }

    /// Full proxy URL for a path (e.g. `/model/info`).
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url.trim_end_matches('/'), path)
    }

    /// Apply the master key as a Bearer token when configured.
    pub fn authed(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.config.master_key {
            Some(key) => rb.bearer_auth(key),
            None => rb,
        }
    }

    /// The underlying HTTP client.
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// The shared Redis pool (cache + usage counters).
    pub fn redis(&self) -> &RedisPool {
        &self.redis
    }
}
