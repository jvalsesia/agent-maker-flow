//! Agent types and field limits.
//!
//! [`Agent`] is the persisted/serialized record; [`AgentInput`] is the request
//! body for create and update (a full replacement of the editable fields). The
//! constants are the PRD field rules — the DB enforces them as a backstop and
//! the service (F04 stage 2) validates against them to produce field-level
//! messages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Max trimmed length of `name`.
pub const NAME_MAX: usize = 64;
/// Max length of the optional `preamble`.
pub const PREAMBLE_MAX: usize = 2000;
/// Max length of `system_prompt`.
pub const SYSTEM_PROMPT_MAX: usize = 32000;
/// Inclusive range for `recent_n`.
pub const RECENT_N_MIN: i32 = 0;
pub const RECENT_N_MAX: i32 = 100;
/// Default `recent_n` when omitted from the request.
pub const RECENT_N_DEFAULT: i32 = 10;
/// Inclusive range for `top_k`.
pub const TOP_K_MIN: i32 = 0;
pub const TOP_K_MAX: i32 = 50;
/// Default `top_k` when omitted from the request.
pub const TOP_K_DEFAULT: i32 = 5;

/// A persisted agent: a reusable LLM behavior profile owned by one user.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub preamble: Option<String>,
    pub system_prompt: String,
    pub provider: String,
    pub model: String,
    pub recent_n: i32,
    pub top_k: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update request body. `recent_n`/`top_k` fall back to their defaults
/// when omitted; all other fields and ranges are validated by the service.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentInput {
    pub name: String,
    #[serde(default)]
    pub preamble: Option<String>,
    pub system_prompt: String,
    pub provider: String,
    pub model: String,
    #[serde(default = "default_recent_n")]
    pub recent_n: i32,
    #[serde(default = "default_top_k")]
    pub top_k: i32,
}

fn default_recent_n() -> i32 {
    RECENT_N_DEFAULT
}

fn default_top_k() -> i32 {
    TOP_K_DEFAULT
}
