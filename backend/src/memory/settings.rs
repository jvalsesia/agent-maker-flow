//! Settings repository: the global per-user embedding model and the optional
//! per-agent semantic profile. Any chosen model is validated against the F03
//! catalog (must exist with `mode == "embedding"`) before it is persisted, and
//! every profile read/write is scoped to the owning user.

use sqlx::PgPool;
use uuid::Uuid;

use super::types::{SemanticProfile, SemanticProfileInput, MEMORY_SCOPES};
use crate::error::AppError;
use crate::gateway::GatewayClient;

fn internal(e: sqlx::Error, ctx: &'static str) -> AppError {
    AppError::Internal(anyhow::anyhow!(e).context(ctx))
}

/// Reject a model that is not an embedding model in the F03 catalog. Propagates
/// `GatewayUnavailable` (GW001) if the catalog cannot be reached; an unknown or
/// non-embedding model yields `InvalidModelForProvider` (GW002).
async fn ensure_embedding_model(gateway: &GatewayClient, model: &str) -> Result<(), AppError> {
    match gateway.find_model(model).await? {
        Some(m) if m.mode == "embedding" => Ok(()),
        _ => Err(AppError::InvalidModelForProvider),
    }
}

/// The caller's global embedding model, or `None` if never set.
pub async fn get_embedding_model(pool: &PgPool, user_id: &str) -> Result<Option<String>, AppError> {
    let model: Option<String> =
        sqlx::query_scalar("SELECT embedding_model FROM user_embedding_settings WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| internal(e, "failed to fetch embedding setting"))?;
    Ok(model)
}

/// Validate and upsert the caller's global embedding model; returns the model.
pub async fn set_embedding_model(
    pool: &PgPool,
    gateway: &GatewayClient,
    user_id: &str,
    model: &str,
) -> Result<String, AppError> {
    ensure_embedding_model(gateway, model).await?;

    sqlx::query(
        "INSERT INTO user_embedding_settings (user_id, embedding_model) \
         VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE \
         SET embedding_model = EXCLUDED.embedding_model, updated_at = now()",
    )
    .bind(user_id)
    .bind(model)
    .execute(pool)
    .await
    .map_err(|e| internal(e, "failed to upsert embedding setting"))?;

    Ok(model.to_string())
}

/// Whether the caller owns the given agent (used to scope profile writes and to
/// validate a memory record's `agent_id` in `store`).
pub(super) async fn agent_owned_by(
    pool: &PgPool,
    user_id: &str,
    agent_id: Uuid,
) -> Result<bool, AppError> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM agents WHERE id = $1 AND owner_id = $2)")
            .bind(agent_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| internal(e, "failed to check agent ownership"))?;
    Ok(exists)
}

/// The caller's semantic profile for an agent, or `None` if absent or the agent
/// is owned by another user.
pub async fn get_semantic_profile(
    pool: &PgPool,
    user_id: &str,
    agent_id: Uuid,
) -> Result<Option<SemanticProfile>, AppError> {
    sqlx::query_as::<_, SemanticProfile>(
        "SELECT agent_id, embedding_model, memory_scope \
         FROM agent_semantic_profiles WHERE agent_id = $1 AND user_id = $2",
    )
    .bind(agent_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| internal(e, "failed to fetch semantic profile"))
}

/// Validate and upsert the semantic profile for one of the caller's agents.
/// `NotFound` if the agent is not owned by the caller.
pub async fn set_semantic_profile(
    pool: &PgPool,
    gateway: &GatewayClient,
    user_id: &str,
    agent_id: Uuid,
    input: &SemanticProfileInput,
) -> Result<SemanticProfile, AppError> {
    if !MEMORY_SCOPES.contains(&input.memory_scope.as_str()) {
        return Err(AppError::Validation {
            code: "MEM_VALIDATION",
            message: "memory_scope must be 'all' or 'own'.".to_string(),
        });
    }
    if !agent_owned_by(pool, user_id, agent_id).await? {
        return Err(AppError::NotFound);
    }
    ensure_embedding_model(gateway, &input.embedding_model).await?;

    sqlx::query_as::<_, SemanticProfile>(
        "INSERT INTO agent_semantic_profiles (agent_id, user_id, embedding_model, memory_scope) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (agent_id) DO UPDATE \
         SET embedding_model = EXCLUDED.embedding_model, \
             memory_scope = EXCLUDED.memory_scope, updated_at = now() \
         RETURNING agent_id, embedding_model, memory_scope",
    )
    .bind(agent_id)
    .bind(user_id)
    .bind(&input.embedding_model)
    .bind(&input.memory_scope)
    .fetch_one(pool)
    .await
    .map_err(|e| internal(e, "failed to upsert semantic profile"))
}

/// Delete the caller's semantic profile for an agent; `true` if one was removed.
pub async fn delete_semantic_profile(
    pool: &PgPool,
    user_id: &str,
    agent_id: Uuid,
) -> Result<bool, AppError> {
    let result =
        sqlx::query("DELETE FROM agent_semantic_profiles WHERE agent_id = $1 AND user_id = $2")
            .bind(agent_id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| internal(e, "failed to delete semantic profile"))?;
    Ok(result.rows_affected() > 0)
}
