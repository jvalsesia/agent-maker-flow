//! Gateway DTOs: the domain types returned to callers and the LiteLLM wire
//! structs used to parse the proxy's responses.

use serde::{Deserialize, Serialize};

/// A discovered provider and how many models it has configured.
#[derive(Debug, Clone, Serialize)]
pub struct Provider {
    pub id: String,
    pub model_count: usize,
}

/// A model offered by a provider. `mode` is `"chat"` or `"embedding"` so the
/// UI can show chat models (F04) and embedding models (F05) separately.
#[derive(Debug, Clone, Serialize)]
pub struct Model {
    pub id: String,
    pub label: String,
    pub mode: String,
}

// --- LiteLLM `/model/info` wire structs ---

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfoResponse {
    #[serde(default)]
    pub data: Vec<ModelInfoEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfoEntry {
    pub model_name: String,
    #[serde(default)]
    pub litellm_params: LiteLlmParams,
    #[serde(default)]
    pub model_info: ModelInfoMeta,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LiteLlmParams {
    /// e.g. `"openai/gpt-4o"` — the `provider/model` form.
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModelInfoMeta {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub litellm_provider: Option<String>,
}
