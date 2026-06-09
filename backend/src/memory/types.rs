//! F05 domain types: the global embedding setting, the per-agent semantic
//! profile, and memory records. The stored `embedding` vector is never sent to
//! clients — list/detail payloads expose `embedding_model` and a `char_count`
//! instead — so the wire type ([`MemoryRecord`]) is distinct from the row read
//! out of the database ([`MemoryRecordRow`]).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maximum length of a memory record's source text (characters).
pub const MEMORY_TEXT_MAX: usize = 8000;

/// Allowed `memory_scope` values for a semantic profile.
pub const MEMORY_SCOPES: [&str; 2] = ["all", "own"];

/// PUT body for the global embedding setting.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingSettingInput {
    pub embedding_model: String,
}

/// A per-agent semantic profile (override model + memory scope).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SemanticProfile {
    pub agent_id: Uuid,
    pub embedding_model: String,
    pub memory_scope: String,
}

/// PUT body for a semantic profile. `memory_scope` defaults to `all`.
#[derive(Debug, Clone, Deserialize)]
pub struct SemanticProfileInput {
    pub embedding_model: String,
    #[serde(default = "default_scope")]
    pub memory_scope: String,
}

fn default_scope() -> String {
    "all".to_string()
}

/// POST/PUT body for a memory record.
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryRecordInput {
    pub text: String,
}

/// A memory record row as stored (the `embedding` vector is selected out only
/// when needed and is not part of this read shape).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemoryRecordRow {
    pub id: Uuid,
    pub text: String,
    pub embedding_model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The client-facing memory record: source text, the model used, a character
/// count, and timestamps — never the raw vector.
#[derive(Debug, Clone, Serialize)]
pub struct MemoryRecord {
    pub id: Uuid,
    pub text: String,
    pub embedding_model: String,
    pub char_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MemoryRecordRow> for MemoryRecord {
    fn from(row: MemoryRecordRow) -> Self {
        let char_count = row.text.chars().count();
        Self {
            id: row.id,
            text: row.text,
            embedding_model: row.embedding_model,
            char_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
