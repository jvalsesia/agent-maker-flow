//! Provider and model discovery via the LiteLLM `/model/info` endpoint.
//!
//! Each configured model is mapped to a `(provider, Model)` pair; providers are
//! derived from `model_info.litellm_provider`, falling back to the `provider/`
//! prefix of `litellm_params.model`. Grouping preserves first-seen order.

use std::collections::HashMap;

use crate::error::AppError;
use crate::gateway::types::{Model, ModelInfoEntry, ModelInfoResponse, Provider};
use crate::gateway::GatewayClient;

/// Determine a model's provider from its `/model/info` entry.
fn provider_of(entry: &ModelInfoEntry) -> String {
    if let Some(p) = &entry.model_info.litellm_provider {
        if !p.is_empty() {
            return p.clone();
        }
    }
    if let Some(model) = &entry.litellm_params.model {
        if let Some((prefix, _)) = model.split_once('/') {
            return prefix.to_string();
        }
    }
    "unknown".to_string()
}

impl GatewayClient {
    /// Fetch every configured model as `(provider, Model)` pairs.
    async fn list_models_all(&self) -> Result<Vec<(String, Model)>, AppError> {
        let request = self.authed(self.http().get(self.url("/model/info")));
        let response = request
            .send()
            .await
            .map_err(|_| AppError::GatewayUnavailable)?;

        if !response.status().is_success() {
            return Err(AppError::GatewayUnavailable);
        }

        let body: ModelInfoResponse = response
            .json()
            .await
            .map_err(|_| AppError::GatewayUnavailable)?;

        Ok(body
            .data
            .into_iter()
            .map(|entry| {
                let provider = provider_of(&entry);
                let mode = entry
                    .model_info
                    .mode
                    .clone()
                    .unwrap_or_else(|| "chat".to_string());
                let model = Model {
                    id: entry.model_name.clone(),
                    label: entry.model_name,
                    mode,
                };
                (provider, model)
            })
            .collect())
    }

    /// List discovered providers with their model counts (first-seen order).
    pub async fn list_providers(&self) -> Result<Vec<Provider>, AppError> {
        let all = self.list_models_all().await?;

        let mut order: Vec<String> = Vec::new();
        let mut counts: HashMap<String, usize> = HashMap::new();
        for (provider, _) in &all {
            if !counts.contains_key(provider) {
                order.push(provider.clone());
            }
            *counts.entry(provider.clone()).or_insert(0) += 1;
        }

        Ok(order
            .into_iter()
            .map(|id| {
                let model_count = counts[&id];
                Provider { id, model_count }
            })
            .collect())
    }

    /// List models for a provider, or `None` when the provider is unknown.
    pub async fn list_models(&self, provider: &str) -> Result<Option<Vec<Model>>, AppError> {
        let models: Vec<Model> = self
            .list_models_all()
            .await?
            .into_iter()
            .filter(|(p, _)| p == provider)
            .map(|(_, m)| m)
            .collect();

        if models.is_empty() {
            Ok(None)
        } else {
            Ok(Some(models))
        }
    }

    /// Find a model by id across all providers, or `None` if not in the
    /// catalog. Used to validate a selected model (e.g. F05 embedding-model
    /// settings) without knowing its provider up front.
    pub async fn find_model(&self, model_id: &str) -> Result<Option<Model>, AppError> {
        Ok(self
            .list_models_all()
            .await?
            .into_iter()
            .find(|(_, m)| m.id == model_id)
            .map(|(_, m)| m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::types::{LiteLlmParams, ModelInfoMeta};

    fn entry(model: &str, provider: Option<&str>) -> ModelInfoEntry {
        ModelInfoEntry {
            model_name: "alias".to_string(),
            litellm_params: LiteLlmParams {
                model: Some(model.to_string()),
            },
            model_info: ModelInfoMeta {
                mode: None,
                litellm_provider: provider.map(|s| s.to_string()),
            },
        }
    }

    #[test]
    fn provider_parsed_from_litellm_model() {
        let e = entry("anthropic/claude-3-5-sonnet", None);
        assert_eq!(provider_of(&e), "anthropic");
    }

    #[test]
    fn provider_prefers_explicit_litellm_provider() {
        let e = entry("openai/gpt-4o", Some("azure"));
        assert_eq!(provider_of(&e), "azure");
    }

    #[test]
    fn provider_falls_back_to_unknown() {
        let e = entry("gpt-4o", None);
        assert_eq!(provider_of(&e), "unknown");
    }
}
