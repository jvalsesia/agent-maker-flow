//! Memory-record repository with embed-on-save orchestration.
//!
//! Create/update resolve the caller's global embedding model, call the F03
//! gateway to embed the text, and only then write the row — a failed embedding
//! is never persisted. All reads/writes are scoped to the owning user.

use pgvector::Vector;
use sqlx::PgPool;
use uuid::Uuid;

use super::settings;
use super::types::{MemoryRecord, MemoryRecordRow, MEMORY_TEXT_MAX};
use crate::error::AppError;
use crate::gateway::types::EmbeddingRequest;
use crate::gateway::GatewayClient;

/// Columns for the client-facing read shape (the `embedding` vector is omitted).
const COLUMNS: &str = "id, text, embedding_model, created_at, updated_at";

fn internal(e: sqlx::Error, ctx: &'static str) -> AppError {
    AppError::Internal(anyhow::anyhow!(e).context(ctx))
}

/// Enforce the 1–8,000 character bound on record text.
fn validate_text(text: &str) -> Result<(), AppError> {
    let len = text.chars().count();
    if len == 0 {
        return Err(AppError::Validation {
            code: "MEM_VALIDATION",
            message: "Memory record text is required.".to_string(),
        });
    }
    if len > MEMORY_TEXT_MAX {
        return Err(AppError::MemoryRecordTooLarge);
    }
    Ok(())
}

/// Resolve the caller's global embedding model and embed `text`. `MEM002` if no
/// global model is configured; gateway errors propagate (GW001/GW002/GW003).
async fn embed_with_global_model(
    pool: &PgPool,
    gateway: &GatewayClient,
    user_id: &str,
    text: &str,
) -> Result<(String, Vector), AppError> {
    let model = settings::get_embedding_model(pool, user_id)
        .await?
        .ok_or(AppError::NoEmbeddingModel)?;

    let result = gateway
        .embed(EmbeddingRequest {
            model: model.clone(),
            input: text.to_string(),
        })
        .await?;

    Ok((model, Vector::from(result.embedding)))
}

/// Create a memory record: validate, embed, then insert (only on embed success).
pub async fn create(
    pool: &PgPool,
    gateway: &GatewayClient,
    user_id: &str,
    text: &str,
) -> Result<MemoryRecord, AppError> {
    validate_text(text)?;
    let (model, embedding) = embed_with_global_model(pool, gateway, user_id, text).await?;

    let row = sqlx::query_as::<_, MemoryRecordRow>(&format!(
        "INSERT INTO memory_records (user_id, text, embedding, embedding_model) \
         VALUES ($1, $2, $3, $4) RETURNING {COLUMNS}"
    ))
    .bind(user_id)
    .bind(text)
    .bind(embedding)
    .bind(&model)
    .fetch_one(pool)
    .await
    .map_err(|e| internal(e, "failed to insert memory record"))?;

    Ok(row.into())
}

/// List the caller's records, newest first (vector omitted).
pub async fn list(pool: &PgPool, user_id: &str) -> Result<Vec<MemoryRecord>, AppError> {
    let rows = sqlx::query_as::<_, MemoryRecordRow>(&format!(
        "SELECT {COLUMNS} FROM memory_records WHERE user_id = $1 ORDER BY created_at DESC"
    ))
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| internal(e, "failed to list memory records"))?;

    Ok(rows.into_iter().map(MemoryRecord::from).collect())
}

/// Distinct embedding models across the caller's records (drives the
/// model-changed warning in the UI).
pub async fn models_in_use(pool: &PgPool, user_id: &str) -> Result<Vec<String>, AppError> {
    sqlx::query_scalar(
        "SELECT DISTINCT embedding_model FROM memory_records WHERE user_id = $1 \
         ORDER BY embedding_model",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| internal(e, "failed to list models in use"))
}

/// Re-embed and update one of the caller's records with the current global
/// model. `NotFound` if absent or owned by another user.
pub async fn update(
    pool: &PgPool,
    gateway: &GatewayClient,
    user_id: &str,
    id: Uuid,
    text: &str,
) -> Result<MemoryRecord, AppError> {
    validate_text(text)?;

    // Confirm ownership before spending an embedding call on a foreign record.
    let owned: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM memory_records WHERE id = $1 AND user_id = $2)")
            .bind(id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| internal(e, "failed to check memory record ownership"))?;
    if !owned {
        return Err(AppError::NotFound);
    }

    let (model, embedding) = embed_with_global_model(pool, gateway, user_id, text).await?;

    let row = sqlx::query_as::<_, MemoryRecordRow>(&format!(
        "UPDATE memory_records \
         SET text = $3, embedding = $4, embedding_model = $5, updated_at = now() \
         WHERE id = $1 AND user_id = $2 RETURNING {COLUMNS}"
    ))
    .bind(id)
    .bind(user_id)
    .bind(text)
    .bind(embedding)
    .bind(&model)
    .fetch_optional(pool)
    .await
    .map_err(|e| internal(e, "failed to update memory record"))?
    .ok_or(AppError::NotFound)?;

    Ok(row.into())
}

/// Delete one of the caller's records; `true` if a row was removed.
pub async fn delete(pool: &PgPool, user_id: &str, id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM memory_records WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| internal(e, "failed to delete memory record"))?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_text() {
        let err = validate_text("").unwrap_err();
        assert_eq!(err.code(), "MEM_VALIDATION");
    }

    #[test]
    fn accepts_text_at_limit() {
        assert!(validate_text(&"a".repeat(MEMORY_TEXT_MAX)).is_ok());
    }

    #[test]
    fn rejects_text_over_limit() {
        let err = validate_text(&"a".repeat(MEMORY_TEXT_MAX + 1)).unwrap_err();
        assert_eq!(err.code(), "MEM001");
    }
}
