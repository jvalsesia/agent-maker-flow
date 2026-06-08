//! Internal completion service: cache → proxy → cache store + counters.

use serde_json::{json, Map, Value};

use crate::error::AppError;
use crate::gateway::cache::cache_key;
use crate::gateway::types::{ChatCompletionResponse, CompletionRequest, CompletionResult};
use crate::gateway::{parse_cost_header, truncate, GatewayClient};

impl GatewayClient {
    /// Run a (non-streaming) chat completion. Serves an exact-match cached
    /// result when present; otherwise calls the proxy, caches the result, and
    /// records usage counters.
    pub async fn complete(&self, req: CompletionRequest) -> Result<CompletionResult, AppError> {
        let key = cache_key(&req);

        if let Some(cached) = self.cache_get(&key).await {
            self.record_cache_hit().await;
            return Ok(cached);
        }

        // Build the OpenAI-compatible body: model + messages + forwarded params.
        let mut body = Map::new();
        body.insert("model".to_string(), json!(req.model));
        body.insert("messages".to_string(), json!(req.messages));
        for (k, v) in &req.params {
            body.insert(k.clone(), v.clone());
        }

        let response = self
            .authed(self.http().post(self.url("/chat/completions")).json(&Value::Object(body)))
            .send()
            .await
            .map_err(|_| AppError::GatewayUnavailable)?;

        let status = response.status();
        let cost = parse_cost_header(&response);

        if !status.is_success() {
            if status.as_u16() == 400 || status.as_u16() == 422 {
                return Err(AppError::InvalidModelForProvider);
            }
            let detail = truncate(response.text().await.unwrap_or_default());
            return Err(AppError::ProviderError(format!(
                "completion failed ({status}): {detail}"
            )));
        }

        let parsed: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| AppError::ProviderError(format!("invalid gateway response: {e}")))?;

        let content = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        let usage = parsed.usage.unwrap_or_default();
        let model = parsed.model.unwrap_or_else(|| req.model.clone());

        let result = CompletionResult {
            content,
            model: model.clone(),
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            cost_usd: cost,
            cached: false,
        };

        self.cache_set(&key, &result).await;
        self.record_usage(
            &model,
            result.prompt_tokens,
            result.completion_tokens,
            result.cost_usd,
        )
        .await;

        Ok(result)
    }
}
