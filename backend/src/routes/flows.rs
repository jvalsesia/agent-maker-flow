//! Flows CRUD + rename endpoints (F08), protected by `require_auth` and scoped
//! to the authenticated caller. Each handler pulls `AuthUser`, delegates to the
//! service, and renders the platform success envelope. List returns lightweight
//! summaries (no graph); open returns the full graph.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::flows::{service, FlowInput, RenameInput};
use crate::state::AppState;

/// `POST /api/v1/flows` — save a new flow for the caller.
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<FlowInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let flow = service::create(&state.db, &user.id, input).await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "status": "success", "data": flow })),
    ))
}

/// `GET /api/v1/flows` — list the caller's flows as summaries (no graph),
/// most recently updated first.
pub async fn list(State(state): State<AppState>, user: AuthUser) -> Result<Json<Value>, AppError> {
    let flows = service::list(&state.db, &user.id).await?;
    Ok(Json(
        json!({ "status": "success", "data": { "flows": flows } }),
    ))
}

/// `GET /api/v1/flows/{id}` — open one of the caller's flows (full graph).
pub async fn get(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let flow = service::get(&state.db, &user.id, id).await?;
    Ok(Json(json!({ "status": "success", "data": flow })))
}

/// `PUT /api/v1/flows/{id}` — re-save the caller's open flow (name + graph).
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<FlowInput>,
) -> Result<Json<Value>, AppError> {
    let flow = service::update(&state.db, &user.id, id, input).await?;
    Ok(Json(json!({ "status": "success", "data": flow })))
}

/// `PATCH /api/v1/flows/{id}` — rename the caller's flow; returns the updated
/// summary (the graph is untouched).
pub async fn rename(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<RenameInput>,
) -> Result<Json<Value>, AppError> {
    let flow = service::rename(&state.db, &user.id, id, input).await?;
    Ok(Json(json!({
        "status": "success",
        "data": {
            "id": flow.id,
            "name": flow.name,
            "created_at": flow.created_at,
            "updated_at": flow.updated_at,
        }
    })))
}

/// `DELETE /api/v1/flows/{id}` — delete the caller's flow; echoes the id.
pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    service::delete(&state.db, &user.id, id).await?;
    Ok(Json(json!({ "status": "success", "data": { "id": id } })))
}
