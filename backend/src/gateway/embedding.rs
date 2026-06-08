//! Internal embedding service: proxy → counters.
//!
//! Embeddings are not cached in F03. On failure the caller (F06 retrieval) is
//! expected to degrade gracefully (skip retrieval for that node).

use serde_json::json;

use crate::error::AppError;
use crate::gateway::types::{EmbeddingRequest, EmbeddingResult, EmbeddingResponse};
use crate::gateway::{parse_cost_header, truncate, GatewayClient};

impl GatewayClient {
    /// Embed a single input string and record usage counters.
    pub async fn embed(&self, req: EmbeddingRequest) -> Result<EmbeddingResult, AppError> {
        let body = json!({ "model": req.model, "input": req.input });

        let response = self
            .authed(self.http().post(self.url("/embeddings")).json(&body))
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
                "embedding failed ({status}): {detail}"
            )));
        }

        let parsed: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| AppError::ProviderError(format!("invalid embedding response: {e}")))?;

        let embedding = parsed
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .unwrap_or_default();
        let usage = parsed.usage.unwrap_or_default();
        let model = parsed.model.unwrap_or_else(|| req.model.clone());

        let result = EmbeddingResult {
            embedding,
            model: model.clone(),
            prompt_tokens: usage.prompt_tokens,
            cost_usd: cost,
        };

        self.record_usage(&model, result.prompt_tokens, 0, result.cost_usd)
            .await;

        Ok(result)
    }
}
