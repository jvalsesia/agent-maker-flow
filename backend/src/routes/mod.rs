//! REST routes mounted under `/api/v1`.
//!
//! Reserve additional mount points here for later features (F02–F10).

use axum::routing::get;
use axum::Router;

use crate::state::AppState;
use crate::sse;

pub mod health;

/// Build the `/api/v1` sub-router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/sse/heartbeat", get(sse::heartbeat))
}
