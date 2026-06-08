//! Provider/model discovery endpoints (F03), protected by `require_auth`.

use axum::extract::{Path, State};
use axum::Json;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

/// `GET /api/v1/providers` — list discovered providers with model counts.
pub async fn list_providers(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let providers = state.gateway.list_providers().await?;
    Ok(Json(json!({
        "status": "success",
        "data": { "providers": providers }
    })))
}

/// `GET /api/v1/providers/{provider}/models` — list models for a provider.
/// Unknown provider → 404.
pub async fn list_models(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Json<Value>, AppError> {
    match state.gateway.list_models(&provider).await? {
        Some(models) => Ok(Json(json!({
            "status": "success",
            "data": { "provider": provider, "models": models }
        }))),
        None => Err(AppError::NotFound),
    }
}
