//! Health endpoint integration test. Requires reachable PostgreSQL and Redis
//! (via DATABASE_URL / REDIS_URL). When they are not available, the test
//! soft-skips so the suite stays green without the docker-compose stack.

use std::net::SocketAddr;

use std::sync::Arc;

use agent_maker_flow_backend::{
    app,
    auth::AuthState,
    cache,
    config::{AppConfig, ClerkConfig, GatewayConfig},
    db,
    gateway::GatewayClient,
    runs::RunRegistry,
    state::AppState,
};

async fn try_state() -> Option<AppState> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    let redis_url = std::env::var("REDIS_URL").ok()?;

    let db = db::init_pool(&database_url).await.ok()?;
    db::run_migrations(&db).await.ok()?;
    cache::verify(&cache::init_pool(&redis_url).ok()?).await.ok()?;
    let redis = cache::init_pool(&redis_url).ok()?;

    let clerk = ClerkConfig {
        issuer: "https://example.clerk.test".to_string(),
        jwks_url: "https://example.clerk.test/.well-known/jwks.json".to_string(),
        authorized_parties: vec!["http://localhost:5173".to_string()],
    };
    let gateway = GatewayConfig {
        base_url: "http://localhost:4000".to_string(),
        master_key: None,
    };

    Some(AppState {
        db,
        redis: redis.clone(),
        config: AppConfig {
            database_url,
            redis_url,
            bind_addr: "127.0.0.1:0".to_string(),
            frontend_origin: "http://localhost:5173".to_string(),
            clerk: clerk.clone(),
            gateway: gateway.clone(),
        },
        auth: AuthState::new(clerk),
        gateway: GatewayClient::new(gateway, redis),
        runs: Arc::new(RunRegistry::new()),
    })
}

#[tokio::test]
async fn health_returns_up_when_dependencies_healthy() {
    let _ = dotenvy::dotenv();

    let Some(state) = try_state().await else {
        eprintln!("SKIP: PostgreSQL/Redis not reachable; health integration test skipped");
        return;
    };

    let router = app::build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let resp = reqwest::get(format!("http://{addr}/api/v1/health"))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "success");
    assert_eq!(body["data"]["service"], "up");
    assert_eq!(body["data"]["database"], "up");
    assert_eq!(body["data"]["cache"], "up");
    assert_eq!(body["data"]["pgvector"], true);
}
