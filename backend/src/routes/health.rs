//! Health endpoint: aggregates service, database, cache, and pgvector status.

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;
use crate::{cache, db};

/// `GET /api/v1/health`
///
/// Returns 200 with the success envelope when all dependencies are healthy,
/// or 503 (`HEALTH001`) when the database or cache is unreachable.
pub async fn health(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let pgvector = db::verify(&state.db)
        .await
        .map_err(|e| AppError::DependencyUnavailable(format!("database unavailable: {e}")))?;

    cache::verify(&state.redis)
        .await
        .map_err(|e| AppError::DependencyUnavailable(format!("cache unavailable: {e}")))?;

    Ok(Json(json!({
        "status": "success",
        "data": {
            "service": "up",
            "database": "up",
            "cache": "up",
            "pgvector": pgvector,
        }
    })))
}
