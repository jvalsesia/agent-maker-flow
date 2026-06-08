//! In-memory JWKS cache for Clerk signing keys.
//!
//! Maps each key id (`kid`) to a `DecodingKey`. Keys are fetched from Clerk's
//! JWKS endpoint, cached with a TTL, and refetched on a TTL miss or when an
//! unknown `kid` is requested (covering Clerk key rotation). Each instance
//! keeps its own copy — keys are small and rotate rarely, so no shared store.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::DecodingKey;

use crate::error::AppError;

/// How long a fetched key set is considered fresh before a refetch.
const JWKS_TTL: Duration = Duration::from_secs(3600);

struct CacheInner {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Option<Instant>,
}

/// Thread-safe cache of Clerk JWKS decoding keys.
pub struct JwksCache {
    jwks_url: String,
    http: reqwest::Client,
    inner: RwLock<CacheInner>,
}

impl JwksCache {
    /// Create an empty cache bound to a JWKS endpoint URL.
    pub fn new(jwks_url: impl Into<String>) -> Self {
        Self {
            jwks_url: jwks_url.into(),
            http: reqwest::Client::new(),
            inner: RwLock::new(CacheInner {
                keys: HashMap::new(),
                fetched_at: None,
            }),
        }
    }

    fn is_fresh(inner: &CacheInner) -> bool {
        inner
            .fetched_at
            .map(|t| t.elapsed() < JWKS_TTL)
            .unwrap_or(false)
    }

    /// Fetch the JWKS document and replace the cached key set.
    /// A network/parse failure surfaces as `AuthServiceUnavailable` (503).
    pub async fn refresh(&self) -> Result<(), AppError> {
        let set: JwkSet = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|_| AppError::AuthServiceUnavailable)?
            .json()
            .await
            .map_err(|_| AppError::AuthServiceUnavailable)?;

        let mut keys = HashMap::new();
        for jwk in &set.keys {
            if let (Some(kid), Ok(key)) = (jwk.common.key_id.clone(), DecodingKey::from_jwk(jwk)) {
                keys.insert(kid, key);
            }
        }

        let mut inner = self.inner.write().expect("jwks cache poisoned");
        inner.keys = keys;
        inner.fetched_at = Some(Instant::now());
        Ok(())
    }

    /// Prime the cache at boot. Failures are logged but not fatal — the first
    /// authenticated request will retry the fetch.
    pub async fn warm(&self) {
        if let Err(e) = self.refresh().await {
            tracing::warn!(error = %e, url = %self.jwks_url, "JWKS warm-up failed; will fetch on first request");
        } else {
            tracing::info!(url = %self.jwks_url, "JWKS cache warmed");
        }
    }

    /// Resolve the decoding key for a `kid`. Returns the cached key when fresh,
    /// otherwise refetches once. Unknown `kid` after a refetch → `Unauthorized`.
    pub async fn decoding_key(&self, kid: &str) -> Result<DecodingKey, AppError> {
        {
            let inner = self.inner.read().expect("jwks cache poisoned");
            if Self::is_fresh(&inner) {
                if let Some(key) = inner.keys.get(kid) {
                    return Ok(key.clone());
                }
            }
        }

        self.refresh().await?;

        let inner = self.inner.read().expect("jwks cache poisoned");
        inner
            .keys
            .get(kid)
            .cloned()
            .ok_or(AppError::Unauthorized)
    }

    /// Insert a decoding key directly and mark the cache fresh. Used to warm a
    /// known key and by tests that inject a locally generated key so token
    /// verification runs without contacting Clerk.
    pub fn insert_key(&self, kid: impl Into<String>, key: DecodingKey) {
        let mut inner = self.inner.write().expect("jwks cache poisoned");
        inner.keys.insert(kid.into(), key);
        inner.fetched_at = Some(Instant::now());
    }
}
