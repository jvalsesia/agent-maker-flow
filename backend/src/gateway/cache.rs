//! Exact-match completion cache (Redis).
//!
//! The cache key is the SHA-256 of the canonical JSON of `(model, messages,
//! params)`, with object keys sorted recursively so semantically identical
//! requests collapse to one key regardless of parameter ordering. Redis errors
//! are treated as a miss/no-op — the cache never fails a request.

use deadpool_redis::redis::AsyncCommands;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::gateway::types::{CompletionRequest, CompletionResult};
use crate::gateway::GatewayClient;

const CACHE_PREFIX: &str = "gw:cache:cmpl:";
const CACHE_TTL_SECS: u64 = 86_400;

/// Recursively sort object keys so serialization is deterministic.
fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut sorted = Map::new();
            for k in keys {
                sorted.insert(k.clone(), canonical(&map[k]));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonical).collect()),
        other => other.clone(),
    }
}

/// Derive the stable cache key for a completion request.
pub fn cache_key(req: &CompletionRequest) -> String {
    let shape = json!({
        "model": req.model,
        "messages": req.messages,
        "params": Value::Object(req.params.clone()),
    });
    let serialized = serde_json::to_string(&canonical(&shape)).unwrap_or_default();

    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    format!("{CACHE_PREFIX}{:x}", hasher.finalize())
}

impl GatewayClient {
    /// Read a cached completion, flipping `cached` to true. Errors → `None`.
    pub async fn cache_get(&self, key: &str) -> Option<CompletionResult> {
        let mut conn = self.redis().get().await.ok()?;
        let raw: Option<String> = conn.get(key).await.ok()?;
        let mut result: CompletionResult = serde_json::from_str(&raw?).ok()?;
        result.cached = true;
        Some(result)
    }

    /// Store a completion result with the 24h TTL. Errors are ignored.
    pub async fn cache_set(&self, key: &str, result: &CompletionResult) {
        if let Ok(mut conn) = self.redis().get().await {
            if let Ok(payload) = serde_json::to_string(result) {
                let _: Result<(), _> = conn.set_ex(key, payload, CACHE_TTL_SECS).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::types::ChatMessage;

    fn req(model: &str, content: &str, params_json: &str) -> CompletionRequest {
        CompletionRequest {
            model: model.to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: content.to_string(),
            }],
            params: serde_json::from_str(params_json).unwrap(),
        }
    }

    #[test]
    fn cache_key_is_order_independent() {
        let a = cache_key(&req("gpt-4o", "hi", r#"{"temperature":0.7,"max_tokens":100}"#));
        let b = cache_key(&req("gpt-4o", "hi", r#"{"max_tokens":100,"temperature":0.7}"#));
        assert_eq!(a, b);
    }

    #[test]
    fn cache_key_differs_on_model_or_messages() {
        let base = cache_key(&req("gpt-4o", "hi", "{}"));
        assert_ne!(base, cache_key(&req("gpt-4o-mini", "hi", "{}")));
        assert_ne!(base, cache_key(&req("gpt-4o", "different", "{}")));
    }
}
