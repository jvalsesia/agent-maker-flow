//! Flow types and field limits.
//!
//! [`Flow`] is the persisted/serialized record (name plus the full graph);
//! [`FlowSummary`] is the lightweight list shape (no graph). [`FlowInput`] is
//! the request body for save (create) and re-save (update); [`RenameInput`] is
//! the name-only body for the dedicated rename. [`FlowGraph`] is the F07 graph
//! persisted verbatim in a single `jsonb` column — `nodes` keep their `id`
//! typed (so the service can check `rootNodeId` membership) and carry every
//! other field through untouched, while `edges` are stored opaquely. The
//! constants are the validation rules the service (F08 stage 2) enforces.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

/// Min trimmed length of `name`.
pub const NAME_MIN: usize = 1;
/// Max trimmed length of `name`.
pub const NAME_MAX: usize = 80;
/// Max serialized size of `graph` (1 MiB). Larger payloads are rejected before
/// they reach the database.
pub const GRAPH_MAX_BYTES: usize = 1024 * 1024;

/// The F07 flow graph, persisted verbatim. Nodes keep their `id` typed and pass
/// every other field through `extra`; edges and the root assignment are stored
/// as-is so the canvas rehydrates exactly as saved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<serde_json::Value>,
    #[serde(rename = "rootNodeId")]
    pub root_node_id: Option<String>,
}

/// A canvas node: an `id` plus its opaque remainder (type, position, agent
/// reference) carried through unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A persisted flow: a name and the full graph owned by one user.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Flow {
    pub id: Uuid,
    pub name: String,
    #[sqlx(json)]
    pub graph: Json<FlowGraph>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A flow without its graph — the shape returned by the list endpoint.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct FlowSummary {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Save (create) / re-save (update) request body. Both endpoints fully replace
/// the editable fields; the service validates the name and graph.
#[derive(Debug, Clone, Deserialize)]
pub struct FlowInput {
    pub name: String,
    pub graph: FlowGraph,
}

/// Rename request body — name only; the graph is untouched.
#[derive(Debug, Clone, Deserialize)]
pub struct RenameInput {
    pub name: String,
}
