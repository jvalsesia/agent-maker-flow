//! REST routes mounted under `/api/v1`.
//!
//! Split into a public router (`health`) and a protected router (`me`,
//! `sse/heartbeat`) guarded by the `require_auth` middleware. Reserve
//! additional mount points here for later features (F03–F10).

use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::Router;

use crate::auth::require_auth;
use crate::state::AppState;
use crate::sse;

pub mod health;
pub mod me;

/// Build the `/api/v1` sub-router. The protected group carries the auth layer;
/// the public group (health) stays reachable without a token.
pub fn router(state: AppState) -> Router<AppState> {
    let public = Router::new().route("/health", get(health::health));

    let protected = Router::new()
        .route("/me", get(me::me))
        .route("/sse/heartbeat", get(sse::heartbeat))
        .route_layer(from_fn_with_state(state, require_auth));

    public.merge(protected)
}
