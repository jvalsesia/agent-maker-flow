//! F09 flow execution HTTP endpoints.
//!
//! - `POST /runs` accepts a graph + prompt, validates, spawns the engine,
//!   and returns the new `runId` so the client can attach to the events
//!   stream.
//! - `GET /runs/{id}/events` is the SSE stream. It honours `Last-Event-ID`
//!   for replay-on-reconnect, and is owner-scoped (other-user runs return
//!   404, never revealing existence).
//! - `GET /runs/{id}` returns the buffered event log + terminal output, used
//!   by F10 to recover after a dropped connection once the run has
//!   finished.

use axum::extract::{Path, State};
use axum::http::header::HeaderMap;
use axum::http::StatusCode;
use axum::response::sse::{KeepAlive, Sse};
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::runs::{model::RunRequest, service};
use crate::state::AppState;

/// `POST /api/v1/runs` — start a run for the supplied graph + prompt.
pub async fn start(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<RunRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let started = service::start(
        state.db.clone(),
        state.gateway.clone(),
        state.runs.clone(),
        &user.id,
        request,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "status": "success",
            "data": {
                "runId": started.run_id,
                "status": "running",
                "nodeCount": started.node_count,
            }
        })),
    ))
}

/// `GET /api/v1/runs/{run_id}/events` — SSE stream: buffered replay then
/// live broadcast until the terminal `run.finished` event lands. Honours
/// `Last-Event-ID` for resume on reconnect.
pub async fn events(
    State(state): State<AppState>,
    user: AuthUser,
    Path(run_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<
    Sse<impl futures::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>>,
    AppError,
> {
    let last_event_id = parse_last_event_id(&headers);
    let subscription = state.runs.subscribe(run_id, &user.id, last_event_id)?;
    let stream = service::into_sse_stream(subscription);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// `GET /api/v1/runs/{run_id}` — owner-scoped snapshot for terminal recovery
/// after a dropped stream.
pub async fn snapshot(
    State(state): State<AppState>,
    user: AuthUser,
    Path(run_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let snap = state.runs.snapshot(run_id, &user.id)?;
    Ok(Json(json!({
        "status": "success",
        "data": {
            "runId": snap.run_id,
            "status": snap.status,
            "output": snap.output,
            "events": snap.events,
        }
    })))
}

/// Parse the `Last-Event-ID` header as a `u64` if present and valid.
fn parse_last_event_id(headers: &HeaderMap) -> Option<u64> {
    headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
}
