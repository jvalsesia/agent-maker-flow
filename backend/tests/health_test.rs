//! Health endpoint integration test. Requires reachable PostgreSQL and Redis
//! (via DATABASE_URL / REDIS_URL). When they are not available, the test
//! soft-skips so the suite stays green without the docker-compose stack.

use std::net::SocketAddr;

use agent_maker_flow_backend::{app, cache, config::AppConfig, db, state::AppState};

async fn try_state() -> Option<AppState> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    let redis_url = std::env::var("REDIS_URL").ok()?;

    let db = db::init_pool(&database_url).await.ok()?;
    db::run_migrations(&db).await.ok()?;
    cache::verify(&cache::init_pool(&redis_url).ok()?).await.ok()?;
    let redis = cache::init_pool(&redis_url).ok()?;

    Some(AppState {
        db,
        redis,
        config: AppConfig {
            database_url,
            redis_url,
            bind_addr: "127.0.0.1:0".to_string(),
            frontend_origin: "http://localhost:5173".to_string(),
        },
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
