//! Agent Maker Flow backend entrypoint.
//!
//! Boot sequence: initialize telemetry and configuration, build shared state,
//! run fail-fast connectivity checks and migrations, then bind and serve.
//! The full `/api/v1` router and SSE base are mounted in stage 3.

use agent_maker_flow_backend::{
    app, auth::AuthState, cache, config::AppConfig, db, gateway::GatewayClient, state::AppState,
    telemetry,
};
use anyhow::Context;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    telemetry::init();

    let config = AppConfig::from_env()?;

    // --- Database: connect, migrate, verify (fail-fast) ---
    let db = db::init_pool(&config.database_url).await.map_err(|e| {
        tracing::error!(dependency = "database", error = %e, "boot check failed");
        e
    })?;
    db::run_migrations(&db).await.map_err(|e| {
        tracing::error!(dependency = "database", error = %e, "migration failed");
        e
    })?;
    let has_vector = db::verify(&db).await.map_err(|e| {
        tracing::error!(dependency = "database", error = %e, "boot check failed");
        e
    })?;
    if !has_vector {
        anyhow::bail!("pgvector extension is not enabled in the database");
    }

    // --- Cache: connect + PING (fail-fast) ---
    let redis = cache::init_pool(&config.redis_url)?;
    cache::verify(&redis).await.map_err(|e| {
        tracing::error!(dependency = "cache", error = %e, "boot check failed");
        e
    })?;

    // --- Auth: build Clerk auth state and warm the JWKS cache (non-fatal) ---
    let auth = AuthState::new(config.clerk.clone());
    auth.jwks.warm().await;

    // --- Gateway: build the LiteLLM client (availability checked at call time) ---
    let gateway = GatewayClient::new(config.gateway.clone(), redis.clone());

    let state = AppState {
        db,
        redis,
        config: config.clone(),
        auth,
        gateway,
    };

    let router = app::build_router(state);

    let listener = TcpListener::bind(&config.bind_addr)
        .await
        .with_context(|| format!("failed to bind {}", config.bind_addr))?;
    tracing::info!(addr = %config.bind_addr, "server listening");

    axum::serve(listener, router).await.context("server error")?;
    Ok(())
}
