//! Agents CRUD endpoints (F04), protected by `require_auth` and scoped to the
//! authenticated caller. Each handler pulls `AuthUser`, delegates to the
//! service, and renders the platform success envelope.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::agents::{service, AgentInput};
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::state::AppState;

/// `POST /api/v1/agents` — create an agent for the caller.
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<AgentInput>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let agent = service::create(&state.db, &state.gateway, &user.id, input).await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "status": "success", "data": agent })),
    ))
}

/// `GET /api/v1/agents` — list the caller's agents, ordered by name.
pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    let agents = service::list(&state.db, &user.id).await?;
    Ok(Json(
        json!({ "status": "success", "data": { "agents": agents } }),
    ))
}

/// `GET /api/v1/agents/{id}` — fetch one of the caller's agents.
pub async fn get(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let agent = service::get(&state.db, &user.id, id).await?;
    Ok(Json(json!({ "status": "success", "data": agent })))
}

/// `PUT /api/v1/agents/{id}` — full-replacement update of the caller's agent.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<AgentInput>,
) -> Result<Json<Value>, AppError> {
    let agent = service::update(&state.db, &state.gateway, &user.id, id, input).await?;
    Ok(Json(json!({ "status": "success", "data": agent })))
}

/// `DELETE /api/v1/agents/{id}` — delete the caller's agent; echoes the id.
pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    service::delete(&state.db, &user.id, id).await?;
    Ok(Json(json!({ "status": "success", "data": { "id": id } })))
}
