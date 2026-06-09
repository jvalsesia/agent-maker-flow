//! Vector retrieval (F06).
//!
//! Embeds an agent's input prompt via the F03 gateway and runs a cosine
//! similarity search over the user's F05 memory vectors, returning the top-K
//! most similar records. The embedding model is resolved from the agent's
//! semantic profile override (F05) if present, otherwise the user's global
//! model; only records in that same model space are comparable, so the search
//! filters by `(user_id, embedding_model)` and reports how many of the user's
//! records were excluded as embedding-space mismatches.
//!
//! `retrieve` is infallible to the caller (F09): every internal failure — no
//! configured model, an embedding error, or a search error — degrades to a
//! flagged `Skipped` outcome with zero records, so a flow run never fails
//! because of retrieval.

use pgvector::Vector;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::settings;
use crate::error::AppError;
use crate::gateway::types::EmbeddingRequest;
use crate::gateway::GatewayClient;

/// Inclusive bounds on the per-node Top-K (mirrors the F04 agent validation).
const TOP_K_MIN: i32 = 0;
const TOP_K_MAX: i32 = 50;

/// A single retrieved memory record and its cosine similarity to the prompt.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RetrievedRecord {
    pub id: Uuid,
    pub text: String,
    /// Cosine similarity `1 - (embedding <=> query)`; higher is more similar.
    pub score: f32,
}

/// Whether retrieval ran or was skipped (and why). A `Skipped` outcome still
/// lets the node proceed without context.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "reason", rename_all = "snake_case")]
pub enum RetrievalStatus {
    Ok,
    Skipped(String),
}

/// The result of a retrieval attempt for one node.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RetrievalOutcome {
    /// Ranked most-similar first; length ≤ the (clamped) Top-K.
    pub records: Vec<RetrievedRecord>,
    /// `records.len()` — surfaced in the F09 execution event.
    pub retrieved_count: usize,
    /// Count of the user's records embedded with a different model (excluded).
    pub excluded_mismatched: usize,
    pub status: RetrievalStatus,
}

impl RetrievalOutcome {
    /// An empty, flagged outcome used for every non-fatal skip path.
    fn skipped(reason: &str) -> Self {
        Self {
            records: Vec::new(),
            retrieved_count: 0,
            excluded_mismatched: 0,
            status: RetrievalStatus::Skipped(reason.to_string()),
        }
    }
}

fn internal(e: sqlx::Error, ctx: &'static str) -> AppError {
    AppError::Internal(anyhow::anyhow!(e).context(ctx))
}

/// Resolve the embedding model for an agent: the per-agent semantic profile
/// override if one exists, otherwise the user's global model.
async fn resolve_model(
    db: &PgPool,
    user_id: &str,
    agent_id: Uuid,
) -> Result<Option<String>, AppError> {
    if let Some(profile) = settings::get_semantic_profile(db, user_id, agent_id).await? {
        return Ok(Some(profile.embedding_model));
    }
    settings::get_embedding_model(db, user_id).await
}

/// Cosine top-K over the user's records in `model`'s space, plus the count of
/// the user's records in *other* model spaces (excluded as mismatches).
async fn search(
    db: &PgPool,
    user_id: &str,
    model: &str,
    embedding: &[f32],
    k: i32,
) -> Result<(Vec<RetrievedRecord>, usize), AppError> {
    let query_vector = Vector::from(embedding.to_vec());

    let rows = sqlx::query_as::<_, (Uuid, String, f64)>(
        "SELECT id, text, 1 - (embedding <=> $1) AS score \
         FROM memory_records \
         WHERE user_id = $2 AND embedding_model = $3 \
         ORDER BY embedding <=> $1 \
         LIMIT $4",
    )
    .bind(query_vector)
    .bind(user_id)
    .bind(model)
    .bind(k as i64)
    .fetch_all(db)
    .await
    .map_err(|e| internal(e, "cosine search failed"))?;

    let records = rows
        .into_iter()
        .map(|(id, text, score)| RetrievedRecord {
            id,
            text,
            score: score as f32,
        })
        .collect();

    let excluded: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM memory_records WHERE user_id = $1 AND embedding_model <> $2",
    )
    .bind(user_id)
    .bind(model)
    .fetch_one(db)
    .await
    .map_err(|e| internal(e, "mismatch count failed"))?;

    Ok((records, excluded as usize))
}

/// Retrieve the top-K memory records most similar to `prompt` for `agent_id`.
/// Infallible: any internal failure yields a flagged `Skipped` outcome.
pub async fn retrieve(
    db: &PgPool,
    gateway: &GatewayClient,
    user_id: &str,
    agent_id: Uuid,
    top_k: i32,
    prompt: &str,
) -> RetrievalOutcome {
    let k = top_k.clamp(TOP_K_MIN, TOP_K_MAX);
    if k == 0 {
        return RetrievalOutcome::skipped("retrieval disabled (top_k = 0)");
    }

    let model = match resolve_model(db, user_id, agent_id).await {
        Ok(Some(model)) => model,
        Ok(None) => return RetrievalOutcome::skipped("no embedding model"),
        Err(_) => return RetrievalOutcome::skipped("search error"),
    };

    let embedding = match gateway
        .embed(EmbeddingRequest {
            model: model.clone(),
            input: prompt.to_string(),
        })
        .await
    {
        Ok(result) => result.embedding,
        Err(_) => return RetrievalOutcome::skipped("embedding failed"),
    };

    match search(db, user_id, &model, &embedding, k).await {
        Ok((records, excluded_mismatched)) => RetrievalOutcome {
            retrieved_count: records.len(),
            records,
            excluded_mismatched,
            status: RetrievalStatus::Ok,
        },
        Err(_) => RetrievalOutcome::skipped("search error"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_top_k_into_range() {
        assert_eq!((-5_i32).clamp(TOP_K_MIN, TOP_K_MAX), 0);
        assert_eq!(10_i32.clamp(TOP_K_MIN, TOP_K_MAX), 10);
        assert_eq!(999_i32.clamp(TOP_K_MIN, TOP_K_MAX), 50);
    }

    #[test]
    fn skipped_outcome_is_empty_and_flagged() {
        let outcome = RetrievalOutcome::skipped("no embedding model");
        assert!(outcome.records.is_empty());
        assert_eq!(outcome.retrieved_count, 0);
        assert_eq!(outcome.excluded_mismatched, 0);
        assert_eq!(
            outcome.status,
            RetrievalStatus::Skipped("no embedding model".to_string())
        );
    }
}
