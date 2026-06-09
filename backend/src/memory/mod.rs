//! Embedding & semantic memory (F05).
//!
//! Per-user embedding-model selection, optional per-agent semantic profiles,
//! and a store of text "memory records" embedded via the F03 gateway and
//! persisted as pgvector vectors. `settings` owns the model/profile rows;
//! `store` owns the embed-on-save record lifecycle. Consumed by F06 (RAG).

pub mod settings;
pub mod store;
pub mod types;

pub use types::{
    EmbeddingSettingInput, MemoryRecord, MemoryRecordInput, SemanticProfile, SemanticProfileInput,
    MEMORY_TEXT_MAX,
};
