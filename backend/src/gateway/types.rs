//! Gateway DTOs: the domain types returned to callers and the LiteLLM wire
//! structs used to parse the proxy's responses.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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

// --- Completion ---

/// A chat message (role + content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// An internal completion request. `params` carries free-form generation
/// parameters (temperature, max_tokens, top_p, …) forwarded to the proxy.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub params: Map<String, Value>,
}

/// The result of a completion, also the cached value (with `cached` flipped to
/// true when served from Redis).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResult {
    pub content: String,
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cost_usd: f64,
    pub cached: bool,
}

// --- Embedding ---

/// An internal embedding request.
#[derive(Debug, Clone)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
}

/// The result of an embedding call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    pub embedding: Vec<f32>,
    pub model: String,
    pub prompt_tokens: u64,
    pub cost_usd: f64,
}

// --- LiteLLM completion/embedding wire structs ---

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub choices: Vec<ChatChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessageOut,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessageOut {
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: u64,
    #[serde(default)]
    pub completion_tokens: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResponse {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub data: Vec<EmbeddingData>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingData {
    #[serde(default)]
    pub embedding: Vec<f32>,
}
