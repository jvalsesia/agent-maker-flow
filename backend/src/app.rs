//! Router assembly: mounts `/api/v1` routes and applies cross-cutting layers.

use axum::http::{HeaderValue, Method};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::routes;
use crate::state::AppState;

/// Build the full application router with CORS, tracing, and shared state.
pub fn build_router(state: AppState) -> Router {
    let cors = build_cors(&state.config.frontend_origin);

    Router::new()
        .nest("/api/v1", routes::router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn build_cors(frontend_origin: &str) -> CorsLayer {
    let origin = frontend_origin
        .parse::<HeaderValue>()
        .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:5173"));

    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers(Any)
}
