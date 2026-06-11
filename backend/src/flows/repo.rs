//! Flows data access (sqlx).
//!
//! Every query is scoped by `owner_id` so a caller can only ever read or mutate
//! their own rows; cross-owner reads/writes simply match zero rows (the service
//! turns that into `NotFound`). The graph is bound and returned as `jsonb` via
//! the `Json<FlowGraph>` wrapper, with no transformation. Validation,
//! uniqueness, and the size cap live in the service layer (F08 stage 2).

use sqlx::types::Json;
use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Flow, FlowInput, FlowSummary};
use crate::error::AppError;

/// Columns selected/returned for a full [`Flow`], in `FromRow` order.
const COLUMNS: &str = "id, name, graph, created_at, updated_at";
/// Columns selected for a [`FlowSummary`] (no graph), in `FromRow` order.
const SUMMARY_COLUMNS: &str = "id, name, created_at, updated_at";

fn internal(e: sqlx::Error, ctx: &'static str) -> AppError {
    AppError::Internal(anyhow::anyhow!(e).context(ctx))
}

/// Insert a new flow owned by `owner_id` and return the stored row.
pub async fn insert(pool: &PgPool, owner_id: &str, input: &FlowInput) -> Result<Flow, AppError> {
    let sql = format!(
        "INSERT INTO flows (owner_id, name, graph) \
         VALUES ($1, $2, $3) \
         RETURNING {COLUMNS}"
    );
    sqlx::query_as::<_, Flow>(&sql)
        .bind(owner_id)
        .bind(&input.name)
        .bind(Json(&input.graph))
        .fetch_one(pool)
        .await
        .map_err(|e| internal(e, "failed to insert flow"))
}

/// Whether the caller already has a flow with this name (case-insensitive).
/// `exclude_id` skips a given row so an update/rename can keep its own name.
pub async fn name_exists(
    pool: &PgPool,
    owner_id: &str,
    name: &str,
    exclude_id: Option<Uuid>,
) -> Result<bool, AppError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM flows \
         WHERE owner_id = $1 AND lower(name) = lower($2) \
         AND ($3::uuid IS NULL OR id <> $3))",
    )
    .bind(owner_id)
    .bind(name)
    .bind(exclude_id)
    .fetch_one(pool)
    .await
    .map_err(|e| internal(e, "failed to check flow name uniqueness"))?;

    Ok(exists)
}

/// List the caller's flows as summaries (no graph), most recently updated first.
pub async fn list_summaries_by_owner(
    pool: &PgPool,
    owner_id: &str,
) -> Result<Vec<FlowSummary>, AppError> {
    let sql = format!(
        "SELECT {SUMMARY_COLUMNS} FROM flows WHERE owner_id = $1 ORDER BY updated_at DESC"
    );
    sqlx::query_as::<_, FlowSummary>(&sql)
        .bind(owner_id)
        .fetch_all(pool)
        .await
        .map_err(|e| internal(e, "failed to list flows"))
}

/// Fetch one of the caller's flows by id; `None` if absent or owned by another.
pub async fn get(pool: &PgPool, owner_id: &str, id: Uuid) -> Result<Option<Flow>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM flows WHERE id = $1 AND owner_id = $2");
    sqlx::query_as::<_, Flow>(&sql)
        .bind(id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| internal(e, "failed to fetch flow"))
}

/// Full-replacement save of the name and graph, refreshing `updated_at`.
/// `None` if the flow is absent or owned by another user.
pub async fn update(
    pool: &PgPool,
    owner_id: &str,
    id: Uuid,
    input: &FlowInput,
) -> Result<Option<Flow>, AppError> {
    let sql = format!(
        "UPDATE flows SET name = $3, graph = $4, updated_at = now() \
         WHERE id = $1 AND owner_id = $2 \
         RETURNING {COLUMNS}"
    );
    sqlx::query_as::<_, Flow>(&sql)
        .bind(id)
        .bind(owner_id)
        .bind(&input.name)
        .bind(Json(&input.graph))
        .fetch_optional(pool)
        .await
        .map_err(|e| internal(e, "failed to update flow"))
}

/// Rename one of the caller's flows, leaving the graph untouched and refreshing
/// `updated_at`. `None` if the flow is absent or owned by another user.
pub async fn rename(
    pool: &PgPool,
    owner_id: &str,
    id: Uuid,
    name: &str,
) -> Result<Option<Flow>, AppError> {
    let sql = format!(
        "UPDATE flows SET name = $3, updated_at = now() \
         WHERE id = $1 AND owner_id = $2 \
         RETURNING {COLUMNS}"
    );
    sqlx::query_as::<_, Flow>(&sql)
        .bind(id)
        .bind(owner_id)
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(|e| internal(e, "failed to rename flow"))
}

/// Delete one of the caller's flows; `true` if a row was removed.
pub async fn delete(pool: &PgPool, owner_id: &str, id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM flows WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(pool)
        .await
        .map_err(|e| internal(e, "failed to delete flow"))?;

    Ok(result.rows_affected() > 0)
}
