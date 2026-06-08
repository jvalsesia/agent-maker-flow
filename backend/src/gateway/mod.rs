//! LLM gateway (F03).
//!
//! A single internal client over the LiteLLM proxy: provider/model discovery,
//! chat completions (with exact-match Redis caching), embeddings, and Redis
//! usage/cost counters. Built once at boot and carried in `AppState`.

use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;

use crate::config::GatewayConfig;

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
