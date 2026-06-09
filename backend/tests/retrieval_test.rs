//! F06 vector-retrieval integration tests.
//!
//! Retrieval is an internal service, so these tests call
//! `memory::retrieval::retrieve` directly (like the F03 completion tests call
//! `gateway.complete`) against a live database and a mock LiteLLM proxy. The
//! mock `/embeddings` encodes the returned vector from the input — a prompt
//! `"vec:1,0,0"` embeds to `[1,0,0]` — so cosine ranking is deterministic;
//! records are seeded through the real F05 `store::create` path. Tests
//! soft-skip when no `DATABASE_URL`/`REDIS_URL` is reachable and each uses a
//! freshly nonced owner so the suite is rerunnable.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_maker_flow_backend::{
    agents::{repo as agents_repo, AgentInput},
    cache,
    config::GatewayConfig,
    db,
    gateway::GatewayClient,
    memory::{retrieval, settings, store, RetrievalStatus, SemanticProfileInput},
};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use deadpool_redis::Pool as RedisPool;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

const MODEL_A: &str = "embed-a";
const MODEL_B: &str = "embed-b";

fn nonce() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}

fn fresh_user(tag: &str) -> String {
    format!("user_rag_{tag}_{}", nonce())
}

// --- Mock LiteLLM proxy: /model/info + input-encoded /embeddings ---

#[derive(Clone)]
struct ProxyState {
    embed_calls: Arc<AtomicUsize>,
    fail_embed: bool,
}

/// Two embedding-mode models so model selection/validation has choices.
async fn mock_model_info() -> Json<Value> {
    Json(json!({
        "data": [
            { "model_name": MODEL_A,
              "litellm_params": { "model": format!("openai/{MODEL_A}") },
              "model_info": { "mode": "embedding", "litellm_provider": "openai" } },
            { "model_name": MODEL_B,
              "litellm_params": { "model": format!("openai/{MODEL_B}") },
              "model_info": { "mode": "embedding", "litellm_provider": "openai" } }
        ]
    }))
}

/// Encode the embedding from the input: `"vec:1,0,0"` → `[1,0,0]`, else a
/// fixed default. Lets tests control cosine ordering precisely.
fn encode_vector(input: &str) -> Vec<f32> {
    input
        .strip_prefix("vec:")
        .map(|rest| rest.split(',').filter_map(|t| t.trim().parse::<f32>().ok()).collect())
        .unwrap_or_else(|| vec![0.1, 0.2, 0.3])
}

async fn mock_embed(State(s): State<ProxyState>, Json(body): Json<Value>) -> axum::response::Response {
    s.embed_calls.fetch_add(1, Ordering::SeqCst);
    if s.fail_embed {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": { "message": "embedding backend exploded" } })),
        )
            .into_response();
    }
    let input = body["input"].as_str().unwrap_or_default();
    let embedding = encode_vector(input);
    let resp = json!({
        "model": body["model"],
        "data": [{ "embedding": embedding, "index": 0 }],
        "usage": { "prompt_tokens": 3, "total_tokens": 3 }
    });
    ([("x-litellm-response-cost", "0.00002")], Json(resp)).into_response()
}

fn mock_proxy(fail_embed: bool) -> (Router, Arc<AtomicUsize>) {
    let state = ProxyState {
        embed_calls: Arc::new(AtomicUsize::new(0)),
        fail_embed,
    };
    let calls = state.embed_calls.clone();
    let router = Router::new()
        .route("/model/info", get(mock_model_info))
        .route("/embeddings", post(mock_embed))
        .with_state(state);
    (router, calls)
}

async fn spawn_router(router: Router) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    addr
}

async fn try_db() -> Option<PgPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = db::init_pool(&url).await.ok()?;
    db::run_migrations(&pool).await.ok()?;
    Some(pool)
}

async fn redis_ready() -> Option<RedisPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let pool = cache::init_pool(&url).ok()?;
    cache::verify(&pool).await.ok()?;
    Some(pool)
}

fn gateway_to(proxy: SocketAddr, redis: RedisPool) -> Arc<GatewayClient> {
    GatewayClient::new(
        GatewayConfig {
            base_url: format!("http://{proxy}"),
            master_key: None,
        },
        redis,
    )
}

/// Build (db, gateway, embed-call-counter) wired to a fresh mock proxy, or
/// `None` if the database/redis are unreachable (soft-skip).
async fn setup(fail_embed: bool) -> Option<(PgPool, Arc<GatewayClient>, Arc<AtomicUsize>)> {
    let db = try_db().await?;
    let redis = redis_ready().await?;
    let (router, calls) = mock_proxy(fail_embed);
    let proxy = spawn_router(router).await;
    Some((db, gateway_to(proxy, redis), calls))
}

/// Set the user's global model then seed a record whose text encodes its vector.
async fn seed(db: &PgPool, gateway: &GatewayClient, user: &str, model: &str, text: &str) {
    settings::set_embedding_model(db, gateway, user, model)
        .await
        .expect("set global model");
    store::create(db, gateway, user, text).await.expect("create memory record");
}

// --- Tests ---

#[tokio::test]
async fn returns_topk_by_cosine_similarity() {
    let Some((db, gateway, _)) = setup(false).await else {
        eprintln!("SKIP: no db/redis; returns_topk_by_cosine_similarity skipped");
        return;
    };
    let user = fresh_user("rank");
    seed(&db, &gateway, &user, MODEL_A, "vec:1,0,0").await;
    seed(&db, &gateway, &user, MODEL_A, "vec:0,1,0").await;
    seed(&db, &gateway, &user, MODEL_A, "vec:0,0,1").await;

    // Query closest to the first record.
    let outcome = retrieval::retrieve(&db, &gateway, &user, Uuid::nil(), 2, "vec:1,0,0").await;

    assert_eq!(outcome.status, RetrievalStatus::Ok);
    assert_eq!(outcome.retrieved_count, 2); // limited by top_k
    assert_eq!(outcome.records.len(), 2);
    assert_eq!(outcome.records[0].text, "vec:1,0,0"); // most similar first
    assert!(outcome.records[0].score >= outcome.records[1].score);
    assert_eq!(outcome.excluded_mismatched, 0);
}

#[tokio::test]
async fn topk_zero_disables_retrieval() {
    let Some((db, gateway, calls)) = setup(false).await else {
        eprintln!("SKIP: no db/redis; topk_zero_disables_retrieval skipped");
        return;
    };
    let user = fresh_user("zero");
    settings::set_embedding_model(&db, &gateway, &user, MODEL_A)
        .await
        .unwrap();
    let before = calls.load(Ordering::SeqCst);

    let outcome = retrieval::retrieve(&db, &gateway, &user, Uuid::nil(), 0, "vec:1,0,0").await;

    assert!(outcome.records.is_empty());
    assert_eq!(
        outcome.status,
        RetrievalStatus::Skipped("retrieval disabled (top_k = 0)".to_string())
    );
    // No embedding call for a disabled node.
    assert_eq!(calls.load(Ordering::SeqCst), before);
}

#[tokio::test]
async fn no_model_configured_skips() {
    let Some((db, gateway, _)) = setup(false).await else {
        eprintln!("SKIP: no db/redis; no_model_configured_skips skipped");
        return;
    };
    let user = fresh_user("nomodel");

    let outcome = retrieval::retrieve(&db, &gateway, &user, Uuid::nil(), 5, "vec:1,0,0").await;

    assert!(outcome.records.is_empty());
    assert_eq!(
        outcome.status,
        RetrievalStatus::Skipped("no embedding model".to_string())
    );
}

#[tokio::test]
async fn embedding_failure_skips_not_errors() {
    let Some((db, gateway, _)) = setup(true).await else {
        eprintln!("SKIP: no db/redis; embedding_failure_skips_not_errors skipped");
        return;
    };
    let user = fresh_user("embedfail");
    // /model/info still works in the fail proxy, so the global model can be set.
    settings::set_embedding_model(&db, &gateway, &user, MODEL_A)
        .await
        .unwrap();

    let outcome = retrieval::retrieve(&db, &gateway, &user, Uuid::nil(), 5, "vec:1,0,0").await;

    assert!(outcome.records.is_empty());
    assert_eq!(
        outcome.status,
        RetrievalStatus::Skipped("embedding failed".to_string())
    );
}

#[tokio::test]
async fn excludes_mismatched_model_records() {
    let Some((db, gateway, _)) = setup(false).await else {
        eprintln!("SKIP: no db/redis; excludes_mismatched_model_records skipped");
        return;
    };
    let user = fresh_user("mismatch");
    seed(&db, &gateway, &user, MODEL_B, "vec:0,0,1").await; // different model space
    seed(&db, &gateway, &user, MODEL_A, "vec:1,0,0").await; // global is now MODEL_A

    let outcome = retrieval::retrieve(&db, &gateway, &user, Uuid::nil(), 5, "vec:1,0,0").await;

    assert_eq!(outcome.status, RetrievalStatus::Ok);
    assert_eq!(outcome.records.len(), 1); // only the MODEL_A record is comparable
    assert_eq!(outcome.records[0].text, "vec:1,0,0");
    assert_eq!(outcome.excluded_mismatched, 1); // the MODEL_B record
}

#[tokio::test]
async fn uses_agent_profile_model_over_global() {
    let Some((db, gateway, _)) = setup(false).await else {
        eprintln!("SKIP: no db/redis; uses_agent_profile_model_over_global skipped");
        return;
    };
    let user = fresh_user("profile");
    seed(&db, &gateway, &user, MODEL_A, "vec:1,0,0").await; // global ends as A after this
    seed(&db, &gateway, &user, MODEL_B, "vec:0,1,0").await; // global ends as B after this
    // Leave the global model as B; the profile will pin this agent to B too,
    // but we assert the profile path is taken by switching global back to A.
    settings::set_embedding_model(&db, &gateway, &user, MODEL_A)
        .await
        .unwrap();

    // An agent owned by the user, with a semantic profile overriding to MODEL_B.
    let agent = agents_repo::insert(
        &db,
        &user,
        &AgentInput {
            name: format!("Agent {}", nonce()),
            preamble: None,
            system_prompt: "Do the thing.".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            recent_n: 10,
            top_k: 5,
        },
    )
    .await
    .unwrap();
    settings::set_semantic_profile(
        &db,
        &gateway,
        &user,
        agent.id,
        &SemanticProfileInput {
            embedding_model: MODEL_B.to_string(),
            memory_scope: "all".to_string(),
        },
    )
    .await
    .unwrap();

    // Global is MODEL_A, but the profile pins this agent to MODEL_B.
    let outcome = retrieval::retrieve(&db, &gateway, &user, agent.id, 5, "vec:0,1,0").await;

    assert_eq!(outcome.status, RetrievalStatus::Ok);
    assert_eq!(outcome.records.len(), 1); // only the MODEL_B record
    assert_eq!(outcome.records[0].text, "vec:0,1,0");
    assert_eq!(outcome.excluded_mismatched, 1); // the MODEL_A record
}

#[tokio::test]
async fn scoped_to_owner() {
    let Some((db, gateway, _)) = setup(false).await else {
        eprintln!("SKIP: no db/redis; scoped_to_owner skipped");
        return;
    };
    let owner = fresh_user("owner");
    let other = fresh_user("other");
    seed(&db, &gateway, &owner, MODEL_A, "vec:1,0,0").await;
    // The other user has the same model but no records of their own.
    settings::set_embedding_model(&db, &gateway, &other, MODEL_A)
        .await
        .unwrap();

    let outcome = retrieval::retrieve(&db, &gateway, &other, Uuid::nil(), 5, "vec:1,0,0").await;

    assert_eq!(outcome.status, RetrievalStatus::Ok);
    assert!(outcome.records.is_empty()); // owner's record is not visible
    assert_eq!(outcome.excluded_mismatched, 0);
}
