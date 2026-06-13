//! Memory-record endpoints (F05), protected by `require_auth` and scoped to the
//! authenticated caller. Create/update embed the text via the F03 gateway
//! before persisting; embed failures surface as the documented gateway errors.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::memory::{store, MemoryRecordInput};
use crate::state::AppState;

/// `GET /api/v1/memory` — the caller's records plus the distinct models in use.
pub async fn list(State(state): State<AppState>, user: AuthUser) -> Result<Json<Value>, AppError> {
    let records = store::list(&state.db, &user.id).await?;
    let models_in_use = store::models_in_use(&state.db, &user.id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": { "records": records, "models_in_use": models_in_use }
    })))
}

/// `POST /api/v1/memory` — embed and store a new record.
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<MemoryRecordInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let record =
        store::create(&state.db, &state.gateway, &user.id, input.agent_id, &input.text).await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "status": "success", "data": record })),
    ))
}

/// `PUT /api/v1/memory/{id}` — re-embed and update one of the caller's records.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<MemoryRecordInput>,
) -> Result<Json<Value>, AppError> {
    let record =
        store::update(&state.db, &state.gateway, &user.id, id, input.agent_id, &input.text).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// `DELETE /api/v1/memory/{id}` — delete one of the caller's records; echoes id.
pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if store::delete(&state.db, &user.id, id).await? {
        Ok(Json(json!({ "status": "success", "data": { "id": id } })))
    } else {
        Err(AppError::NotFound)
    }
}
