//! Embedding settings & per-agent semantic profile endpoints (F05), protected
//! by `require_auth` and scoped to the authenticated caller.

use axum::extract::{Path, State};
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::memory::{settings, EmbeddingSettingInput, SemanticProfileInput};
use crate::state::AppState;

/// `GET /api/v1/settings/embedding` — the caller's global model, or null.
pub async fn get_embedding(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    let model = settings::get_embedding_model(&state.db, &user.id).await?;
    Ok(Json(json!({ "status": "success", "data": { "embedding_model": model } })))
}

/// `PUT /api/v1/settings/embedding` — validate and set the global model.
pub async fn set_embedding(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<EmbeddingSettingInput>,
) -> Result<Json<Value>, AppError> {
    let model =
        settings::set_embedding_model(&state.db, &state.gateway, &user.id, &input.embedding_model)
            .await?;
    Ok(Json(json!({ "status": "success", "data": { "embedding_model": model } })))
}

/// `GET /api/v1/agents/{id}/semantic-profile` — the caller's profile for an
/// agent; 404 if absent or the agent belongs to another user.
pub async fn get_semantic_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let profile = settings::get_semantic_profile(&state.db, &user.id, agent_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(json!({ "status": "success", "data": profile })))
}

/// `PUT /api/v1/agents/{id}/semantic-profile` — validate and upsert the profile.
pub async fn set_semantic_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Path(agent_id): Path<Uuid>,
    Json(input): Json<SemanticProfileInput>,
) -> Result<Json<Value>, AppError> {
    let profile =
        settings::set_semantic_profile(&state.db, &state.gateway, &user.id, agent_id, &input)
            .await?;
    Ok(Json(json!({ "status": "success", "data": profile })))
}

/// `DELETE /api/v1/agents/{id}/semantic-profile` — remove the profile; 404 if
/// none exists for the caller.
pub async fn delete_semantic_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if settings::delete_semantic_profile(&state.db, &user.id, agent_id).await? {
        Ok(Json(json!({ "status": "success", "data": { "agent_id": agent_id } })))
    } else {
        Err(AppError::NotFound)
    }
}
