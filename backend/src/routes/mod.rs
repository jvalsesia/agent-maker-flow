//! REST routes mounted under `/api/v1`.
//!
//! Split into a public router (`health`) and a protected router (`me`,
//! `sse/heartbeat`) guarded by the `require_auth` middleware. Reserve
//! additional mount points here for later features (F03–F10).

use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::Router;

use crate::auth::require_auth;
use crate::state::AppState;
use crate::sse;

pub mod agents;
pub mod health;
pub mod me;
pub mod providers;

/// Build the `/api/v1` sub-router. The protected group carries the auth layer;
/// the public group (health) stays reachable without a token.
pub fn router(state: AppState) -> Router<AppState> {
    let public = Router::new().route("/health", get(health::health));

    let protected = Router::new()
        .route("/me", get(me::me))
        .route("/providers", get(providers::list_providers))
        .route("/providers/{provider}/models", get(providers::list_models))
        .route("/agents", post(agents::create).get(agents::list))
        .route(
            "/agents/{id}",
            get(agents::get).put(agents::update).delete(agents::delete),
        )
        .route("/sse/heartbeat", get(sse::heartbeat))
        .route_layer(from_fn_with_state(state, require_auth));

    public.merge(protected)
}
