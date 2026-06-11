//! REST routes mounted under `/api/v1`.
//!
//! Split into a public router (`health`) and a protected router (`me`,
//! `sse/heartbeat`) guarded by the `require_auth` middleware. Reserve
//! additional mount points here for later features (F03–F10).

use axum::middleware::from_fn_with_state;
use axum::routing::{get, post, put};
use axum::Router;

use crate::auth::require_auth;
use crate::sse;
use crate::state::AppState;

pub mod agents;
pub mod flows;
pub mod health;
pub mod me;
pub mod memory;
pub mod providers;
pub mod runs;
pub mod settings;

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
        .route("/flows", post(flows::create).get(flows::list))
        .route(
            "/flows/{id}",
            get(flows::get)
                .put(flows::update)
                .patch(flows::rename)
                .delete(flows::delete),
        )
        .route(
            "/agents/{id}/semantic-profile",
            get(settings::get_semantic_profile)
                .put(settings::set_semantic_profile)
                .delete(settings::delete_semantic_profile),
        )
        .route(
            "/settings/embedding",
            get(settings::get_embedding).put(settings::set_embedding),
        )
        .route("/memory", get(memory::list).post(memory::create))
        .route("/memory/{id}", put(memory::update).delete(memory::delete))
        .route("/runs", post(runs::start))
        .route("/runs/{run_id}", get(runs::snapshot))
        .route("/runs/{run_id}/events", get(runs::events))
        .route("/sse/heartbeat", get(sse::heartbeat))
        .route_layer(from_fn_with_state(state, require_auth));

    public.merge(protected)
}
