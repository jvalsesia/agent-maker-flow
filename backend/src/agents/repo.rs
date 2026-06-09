//! Agents data access (sqlx).
//!
//! Every query is scoped by `owner_id` so a caller can only ever read or mutate
//! their own rows; cross-owner reads/writes simply match zero rows (the service
//! turns that into `NotFound`). Validation, uniqueness, and provider/model
//! checks live in the service layer (F04 stage 2).

use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Agent, AgentInput};
use crate::error::AppError;

/// Columns selected/returned for an [`Agent`], in `FromRow` order.
const COLUMNS: &str = "id, name, preamble, system_prompt, provider, model, \
                       recent_n, top_k, created_at, updated_at";

fn internal(e: sqlx::Error, ctx: &'static str) -> AppError {
    AppError::Internal(anyhow::anyhow!(e).context(ctx))
}

/// Insert a new agent owned by `owner_id` and return the stored row.
pub async fn insert(pool: &PgPool, owner_id: &str, input: &AgentInput) -> Result<Agent, AppError> {
    let sql = format!(
        "INSERT INTO agents \
         (owner_id, name, preamble, system_prompt, provider, model, recent_n, top_k) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING {COLUMNS}"
    );
    sqlx::query_as::<_, Agent>(&sql)
        .bind(owner_id)
        .bind(&input.name)
        .bind(&input.preamble)
        .bind(&input.system_prompt)
        .bind(&input.provider)
        .bind(&input.model)
        .bind(input.recent_n)
        .bind(input.top_k)
        .fetch_one(pool)
        .await
        .map_err(|e| internal(e, "failed to insert agent"))
}

/// Whether the caller already has an agent with this name (case-insensitive).
/// `exclude_id` skips a given row so an update can keep its own name.
pub async fn name_exists(
    pool: &PgPool,
    owner_id: &str,
    name: &str,
    exclude_id: Option<Uuid>,
) -> Result<bool, AppError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM agents \
         WHERE owner_id = $1 AND lower(name) = lower($2) \
         AND ($3::uuid IS NULL OR id <> $3))",
    )
    .bind(owner_id)
    .bind(name)
    .bind(exclude_id)
    .fetch_one(pool)
    .await
    .map_err(|e| internal(e, "failed to check agent name uniqueness"))?;

    Ok(exists)
}

/// List the caller's agents, ordered by name ascending.
pub async fn list_by_owner(pool: &PgPool, owner_id: &str) -> Result<Vec<Agent>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM agents WHERE owner_id = $1 ORDER BY name ASC");
    sqlx::query_as::<_, Agent>(&sql)
        .bind(owner_id)
        .fetch_all(pool)
        .await
        .map_err(|e| internal(e, "failed to list agents"))
}

/// Fetch one of the caller's agents by id; `None` if absent or owned by another.
pub async fn get(pool: &PgPool, owner_id: &str, id: Uuid) -> Result<Option<Agent>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM agents WHERE id = $1 AND owner_id = $2");
    sqlx::query_as::<_, Agent>(&sql)
        .bind(id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| internal(e, "failed to fetch agent"))
}

/// Full-replacement update of the editable fields, refreshing `updated_at`.
/// `None` if the agent is absent or owned by another user.
pub async fn update(
    pool: &PgPool,
    owner_id: &str,
    id: Uuid,
    input: &AgentInput,
) -> Result<Option<Agent>, AppError> {
    let sql = format!(
        "UPDATE agents SET \
         name = $3, preamble = $4, system_prompt = $5, provider = $6, model = $7, \
         recent_n = $8, top_k = $9, updated_at = now() \
         WHERE id = $1 AND owner_id = $2 \
         RETURNING {COLUMNS}"
    );
    sqlx::query_as::<_, Agent>(&sql)
        .bind(id)
        .bind(owner_id)
        .bind(&input.name)
        .bind(&input.preamble)
        .bind(&input.system_prompt)
        .bind(&input.provider)
        .bind(&input.model)
        .bind(input.recent_n)
        .bind(input.top_k)
        .fetch_optional(pool)
        .await
        .map_err(|e| internal(e, "failed to update agent"))
}

/// Delete one of the caller's agents; `true` if a row was removed.
pub async fn delete(pool: &PgPool, owner_id: &str, id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM agents WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(pool)
        .await
        .map_err(|e| internal(e, "failed to delete agent"))?;

    Ok(result.rows_affected() > 0)
}
