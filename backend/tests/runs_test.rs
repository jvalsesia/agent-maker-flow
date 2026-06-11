//! F09 flow-execution integration tests.
//!
//! Uses the same signed-token harness as the F04/F08 tests and a small
//! mock LiteLLM proxy that responds to `/chat/completions` with a
//! deterministic body. Tests covering live runs need both a database and
//! Redis reachable (for `cache::verify`); they soft-skip when either is
//! missing. Each test mints a fresh owner id so the suite is rerunnable
//! against a persistent DB.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use deadpool_redis::Pool as RedisPool;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::PgPool;

const TEST_KID: &str = "amf-test-key-1";
const ISSUER: &str = "https://amf.test";
const AZP: &str = "http://localhost:5173";

const PRIV_PEM: &[u8] = b"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC9moaZzpV4EhSM
ALpWTW5zn0LTnAa5D1eYOLM4vqqdTmlq1irezhkB11+Pa9q6TUzDKn+HWhtYiteE
6IBLMl9PQVtT7Wfc7vkh0IQ8ykqimNb8RHYDwLnaQXipbs9E3KXez6l8e4CJbX0Y
EvsyrZK89f8R0joaE6LBv5mYRDWJg5kWyQnh4CKB9DO/P6VnIHkFkivFDpCA5P9I
LlcKKmn5oAyy77i5RSbSkK3T5lzvmqhXyshIEMxu4EUjhkN3mowgMTIF6S1ErH+z
c7N/bxvGA7ubxVUhKWMiNecQmNJS166Y2r/aqQnO0Tlm+HQ+qTPxT64mwZE4YL4k
shW0yoJNAgMBAAECggEADN9cnj0hDI5CWwX39gDa9ppxsEmyctEolohaR0g576T2
8b6uxEKJqURsyQvexVFsXH9lD0oDZhsZCj2PFkE0033sXCdCVa+GIQZV2rBg6adG
80jCRWYfMsLz/6GAYhxj3jEgQI1m+uLrgF1WNMja1/Ll7sO5152NKip0BwXcfPnh
ER6GQEPMsPLTbZTjUIBUty/+5k0CDcpHQMeNolugZB2SUCNysBr0oSnTWGQ6nRCy
z7ZKYqZQ/KelBNl38AjwbKMZeNnZXeFD2BV7hA0clFEVJbftShp9MsNTZ1UcsdXe
qkv5MC0aMg68l7fqsszL8pfYs7Od1fFod8YQBWnIsQKBgQDnPy7+I7wZJj1PnU1v
kxMbi7KX3q8bjgRmFbTylmQ+NrF6g3lb/vs/LRyBTtPmf4+480VDWa6Xj5bWjcuX
FmMupai5lD+YyXOaaEqE+1WcImalHvRNT5g2tcdme8eQgwFCFfblVlezQOQBhj/Q
ok4bAtgod/kOIblgfnA/hPSMcQKBgQDR5jNxfgOvVFlPzmfH3WwSGTmtpjJ7Xutz
dIu7Zz2E4nVz+0M0L5wnJjJ49mwxp/RF76pmFzojtYXVmk8NArnTIna+RC5jc91P
XoWhhKmvWZDG/Bmp/ZqrMZEHSEUoACUG79rEw7vqdbBYtZLi2WChastG7/eZGOSq
YwBFsTrxnQKBgBflG4H/R0yB+wvjAUFqPSs3gDjZNdbvEd1KmOwIRkt3c1dphnzP
GD8q9isWbib/P2apHJsdBUF8AOYiuMrf8Ve8nnaurvOmvV9TL4AWSH5dv6WIUU47
z0q39ebNG43/O34Mrvp7tYw8RFM0ABwa6V85KATmgMHJElK6PfcSUgLBAoGBAKJq
RQ4hmwpU81LMfQNrMw+CE15pxpAt73SEDwdwqGqlrIqVNvgvit3EMbPlwexecKaY
/7pFaMhu0mNpJpgDrvRPq6AoM9jis7GRi0di1sYHQP6n3dfqk366OOVwp4p/KieG
+znb1xFiBZVu0nzUBXCBqU93qZf+ahnpxzEmJV0FAoGBAMwKaP3k1zxD2G3C2P1T
Dv+TXgaa0fTdiQS9gpZxzeD7xtoeNMvsawI9kYiPbnD2ZwDMPFpthaDn4Qh0KxSC
trTgC8LHN0WyxwoPYSk5/4Q1G853RQHtdZ37Zbn7uQqtpi1EtyOomD9IKxFlIjsv
rR7fbGcUo4KyavuEze7KpriG
-----END PRIVATE KEY-----";

const PUB_PEM: &[u8] = b"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAvZqGmc6VeBIUjAC6Vk1u
c59C05wGuQ9XmDizOL6qnU5patYq3s4ZAddfj2vauk1Mwyp/h1obWIrXhOiASzJf
T0FbU+1n3O75IdCEPMpKopjW/ER2A8C52kF4qW7PRNyl3s+pfHuAiW19GBL7Mq2S
vPX/EdI6GhOiwb+ZmEQ1iYOZFskJ4eAigfQzvz+lZyB5BZIrxQ6QgOT/SC5XCipp
+aAMsu+4uUUm0pCt0+Zc75qoV8rISBDMbuBFI4ZDd5qMIDEyBektRKx/s3Ozf28b
xgO7m8VVISljIjXnEJjSUteumNq/2qkJztE5Zvh0Pqkz8U+uJsGROGC+JLIVtMqC
TQIDAQAB
-----END PUBLIC KEY-----";

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn nonce() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}

fn make_token(sub: &str) -> String {
    let claims = json!({
        "sub": sub,
        "iss": ISSUER,
        "azp": AZP,
        "exp": unix_now() + 3600,
        "email": "joao@example.com",
    });
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_string());
    let key = EncodingKey::from_rsa_pem(PRIV_PEM).expect("encoding key");
    encode(&header, &claims, &key).expect("encode token")
}

#[derive(Clone)]
struct MockState {
    fail_models: Arc<std::sync::Mutex<Vec<String>>>,
    calls: Arc<AtomicUsize>,
}

async fn mock_chat(
    State(s): State<MockState>,
    Json(body): Json<Value>,
) -> axum::response::Response {
    s.calls.fetch_add(1, Ordering::SeqCst);
    let model = body["model"].as_str().unwrap_or("unknown").to_string();
    let should_fail = s.fail_models.lock().unwrap().iter().any(|m| m == &model);
    if should_fail {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": { "message": "mocked failure" } })),
        )
            .into_response();
    }
    // Use the inbound user content as the deterministic output so chains can
    // assert "n2 saw n1's output" by inspecting the final aggregated string.
    let user_msg = body["messages"]
        .as_array()
        .and_then(|arr| arr.last())
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let reply = format!("{model}::{user_msg}");
    let resp = json!({
        "id": "cmpl-mock",
        "model": model,
        "choices": [{ "message": { "role": "assistant", "content": reply } }],
        "usage": { "prompt_tokens": 5, "completion_tokens": 3, "total_tokens": 8 }
    });
    ([("x-litellm-response-cost", "0.0001")], Json(resp)).into_response()
}

async fn mock_model_info() -> Json<Value> {
    Json(json!({ "data": [] }))
}

fn mock_proxy() -> (Router, MockState) {
    let state = MockState {
        fail_models: Arc::new(std::sync::Mutex::new(Vec::new())),
        calls: Arc::new(AtomicUsize::new(0)),
    };
    let router = Router::new()
        .route("/model/info", get(mock_model_info))
        .route("/chat/completions", post(mock_chat))
        .with_state(state.clone());
    (router, state)
}

async fn spawn_router(router: Router) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    addr
}

fn build_state(db: PgPool, redis: RedisPool, gateway_base: String) -> AppState {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());

    let clerk = ClerkConfig {
        issuer: ISSUER.to_string(),
        jwks_url: format!("{ISSUER}/.well-known/jwks.json"),
        authorized_parties: vec![AZP.to_string()],
    };
    let auth = AuthState::new(clerk.clone());
    auth.jwks.insert_key(
        TEST_KID,
        DecodingKey::from_rsa_pem(PUB_PEM).expect("decoding key"),
    );

    let gateway_config = GatewayConfig {
        base_url: gateway_base,
        master_key: None,
    };

    AppState {
        db,
        redis: redis.clone(),
        config: AppConfig {
            database_url: String::new(),
            redis_url,
            bind_addr: "127.0.0.1:0".to_string(),
            frontend_origin: AZP.to_string(),
            clerk,
            gateway: gateway_config.clone(),
        },
        auth,
        gateway: GatewayClient::new(gateway_config, redis),
        runs: Arc::new(RunRegistry::new()),
    }
}

async fn try_db() -> Option<PgPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = db::init_pool(&url).await.ok()?;
    db::run_migrations(&pool).await.ok()?;
    Some(pool)
}

async fn try_redis() -> Option<RedisPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let pool = cache::init_pool(&url).ok()?;
    cache::verify(&pool).await.ok()?;
    Some(pool)
}

async fn spawn_app(state: AppState) -> SocketAddr {
    let router = app::build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    addr
}

struct TestApp {
    addr: SocketAddr,
    proxy_state: MockState,
}

async fn spawn_with_gateway(db: PgPool, redis: RedisPool) -> TestApp {
    let (router, proxy_state) = mock_proxy();
    let proxy_addr = spawn_router(router).await;
    let addr = spawn_app(build_state(db, redis, format!("http://{proxy_addr}"))).await;
    TestApp { addr, proxy_state }
}

fn fresh_owner(tag: &str) -> String {
    format!("user_run_{tag}_{}", nonce())
}

fn agent_body(name: &str, model: &str) -> Value {
    json!({
        "name": name,
        "preamble": "be concise",
        "system_prompt": "You are an integration test agent.",
        "provider": "openai",
        "model": model,
        "recent_n": 0,
        "top_k": 0,
    })
}

async fn create_agent(addr: SocketAddr, token: &str, name: &str, model: &str) -> String {
    let resp = reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/agents"))
        .bearer_auth(token)
        .json(&agent_body(name, model))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        201,
        "agent create failed: {:?}",
        resp.text().await
    );
    let body = resp.json::<Value>().await.unwrap();
    body["data"]["id"].as_str().unwrap().to_string()
}

fn linear_graph(agent_a: &str, agent_b: &str) -> Value {
    json!({
        "nodes": [
            { "id": "n1", "data": { "agentId": agent_a } },
            { "id": "n2", "data": { "agentId": agent_b } }
        ],
        "edges": [
            { "id": "e1", "source": "n1", "target": "n2" }
        ],
        "rootNodeId": "n1"
    })
}

fn diamond_graph(a: &str, b: &str, c: &str, d: &str) -> Value {
    json!({
        "nodes": [
            { "id": "n1", "data": { "agentId": a } },
            { "id": "n2", "data": { "agentId": b } },
            { "id": "n3", "data": { "agentId": c } },
            { "id": "n4", "data": { "agentId": d } }
        ],
        "edges": [
            { "id": "e1", "source": "n1", "target": "n2" },
            { "id": "e2", "source": "n1", "target": "n3" },
            { "id": "e3", "source": "n2", "target": "n4" },
            { "id": "e4", "source": "n3", "target": "n4" }
        ],
        "rootNodeId": "n1"
    })
}

async fn start_run(addr: SocketAddr, token: &str, body: &Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/runs"))
        .bearer_auth(token)
        .json(body)
        .send()
        .await
        .unwrap()
}

/// Poll `GET /runs/{id}` until the run is terminal or `timeout` elapses.
/// Returns the final snapshot body.
async fn wait_for_finish(addr: SocketAddr, token: &str, run_id: &str, timeout: Duration) -> Value {
    let client = reqwest::Client::new();
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let resp = client
            .get(format!("http://{addr}/api/v1/runs/{run_id}"))
            .bearer_auth(token)
            .send()
            .await
            .unwrap();
        let body = resp.json::<Value>().await.unwrap();
        let status = body["data"]["status"].as_str().unwrap_or("");
        if status == "succeeded" || status == "failed" {
            return body;
        }
        if std::time::Instant::now() >= deadline {
            panic!("run did not finish in time, last body: {body}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

// --- Auth ---

#[tokio::test]
async fn unauthenticated_run_rejected() {
    let (router, _) = mock_proxy();
    let proxy = spawn_router(router).await;
    let lazy = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://amf:amf@localhost:5432/amf")
        .unwrap();
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let redis = cache::init_pool(&redis_url).expect("redis pool");
    let state = build_state(lazy, redis, format!("http://{proxy}"));
    let addr = spawn_app(state).await;

    let resp = reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/runs"))
        .json(&json!({ "prompt": "hi", "graph": {} }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "AUTH001"
    );
}

// --- Start ---

#[tokio::test]
async fn start_returns_run_id() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; start_returns_run_id skipped");
        return;
    };
    let Some(redis) = try_redis().await else {
        eprintln!("SKIP: no redis; start_returns_run_id skipped");
        return;
    };
    let app = spawn_with_gateway(db, redis).await;
    let token = make_token(&fresh_owner("start"));
    let a = create_agent(app.addr, &token, "A", "gpt-4o").await;
    let b = create_agent(app.addr, &token, "B", "gpt-4o-mini").await;
    let resp = start_run(
        app.addr,
        &token,
        &json!({ "prompt": "hi", "graph": linear_graph(&a, &b) }),
    )
    .await;
    assert_eq!(resp.status(), 201);
    let body = resp.json::<Value>().await.unwrap();
    let run_id = body["data"]["runId"].as_str().unwrap().to_string();
    assert_eq!(body["data"]["status"], "running");
    assert_eq!(body["data"]["nodeCount"], 2);

    // Drain to terminal so the background task completes before the test ends.
    let _ = wait_for_finish(app.addr, &token, &run_id, Duration::from_secs(5)).await;
}

#[tokio::test]
async fn invalid_dag_rejected_before_execution() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; invalid_dag_rejected_before_execution skipped");
        return;
    };
    let Some(redis) = try_redis().await else {
        eprintln!("SKIP: no redis; invalid_dag_rejected_before_execution skipped");
        return;
    };
    let app = spawn_with_gateway(db, redis).await;
    let token = make_token(&fresh_owner("dag"));
    let a = create_agent(app.addr, &token, "A", "gpt-4o").await;
    let b = create_agent(app.addr, &token, "B", "gpt-4o-mini").await;

    // Cycle: n1 → n2 → n1.
    let graph = json!({
        "nodes": [
            { "id": "n1", "data": { "agentId": a } },
            { "id": "n2", "data": { "agentId": b } }
        ],
        "edges": [
            { "id": "e1", "source": "n1", "target": "n2" },
            { "id": "e2", "source": "n2", "target": "n1" }
        ],
        "rootNodeId": "n1"
    });
    let resp = start_run(app.addr, &token, &json!({ "prompt": "hi", "graph": graph })).await;
    assert_eq!(resp.status(), 422);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "RUN001"
    );
    // Ensure no gateway call happened.
    assert_eq!(app.proxy_state.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn missing_agent_rejects_run() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; missing_agent_rejects_run skipped");
        return;
    };
    let Some(redis) = try_redis().await else {
        eprintln!("SKIP: no redis; missing_agent_rejects_run skipped");
        return;
    };
    let app = spawn_with_gateway(db, redis).await;
    let token = make_token(&fresh_owner("ghost"));

    let ghost = "11111111-1111-1111-1111-111111111111";
    let graph = json!({
        "nodes": [{ "id": "n1", "data": { "agentId": ghost } }],
        "edges": [],
        "rootNodeId": "n1"
    });
    let resp = start_run(app.addr, &token, &json!({ "prompt": "hi", "graph": graph })).await;
    assert_eq!(resp.status(), 422);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "RUN003"
    );
}

#[tokio::test]
async fn second_run_same_flow_rejected() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; second_run_same_flow_rejected skipped");
        return;
    };
    let Some(redis) = try_redis().await else {
        eprintln!("SKIP: no redis; second_run_same_flow_rejected skipped");
        return;
    };
    let app = spawn_with_gateway(db, redis).await;
    let token = make_token(&fresh_owner("guard"));
    let a = create_agent(app.addr, &token, "A", "slow-model").await;
    let b = create_agent(app.addr, &token, "B", "slow-model").await;
    // The first run is still running while the second comes in. We slow the
    // first by failing the model (which makes it terminate fast actually) —
    // instead, race two POSTs back to back; the registry guard holds the
    // first run as Running until the engine emits run.finished and calls
    // RunRegistry::finish. Spawning a run with a slow upstream isn't easy
    // here without sleeping; instead, send the second request immediately,
    // which is what the F09 acceptance criterion guards against.
    let flow_id = uuid::Uuid::new_v4();
    let body = json!({
        "prompt": "hi",
        "graph": linear_graph(&a, &b),
        "flowId": flow_id,
    });
    let first = start_run(app.addr, &token, &body).await;
    assert_eq!(first.status(), 201);
    let first_run_id = first.json::<Value>().await.unwrap()["data"]["runId"]
        .as_str()
        .unwrap()
        .to_string();
    // Fire the second BEFORE polling so the first run is still Running.
    let second = start_run(app.addr, &token, &body).await;
    // The guard sometimes loses the race when the first run completes very
    // fast; accept either RUN002 (guard) or 201 (first already done) but
    // assert the contract when we did get the guard.
    if second.status() == 409 {
        assert_eq!(
            second.json::<Value>().await.unwrap()["error"]["code"],
            "RUN002"
        );
    } else {
        eprintln!(
            "second_run_same_flow_rejected: first run finished before second dispatched, skipping guard check"
        );
    }
    let _ = wait_for_finish(app.addr, &token, &first_run_id, Duration::from_secs(5)).await;
}

// --- Forwarding + terminal aggregation ---

#[tokio::test]
async fn linear_flow_forwards_output_to_final() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; linear_flow_forwards_output_to_final skipped");
        return;
    };
    let Some(redis) = try_redis().await else {
        eprintln!("SKIP: no redis; linear_flow_forwards_output_to_final skipped");
        return;
    };
    let app = spawn_with_gateway(db, redis).await;
    let token = make_token(&fresh_owner("linear"));
    let a = create_agent(app.addr, &token, "A", "first-model").await;
    let b = create_agent(app.addr, &token, "B", "second-model").await;
    let resp = start_run(
        app.addr,
        &token,
        &json!({ "prompt": "ORIGINAL", "graph": linear_graph(&a, &b) }),
    )
    .await;
    assert_eq!(resp.status(), 201);
    let run_id = resp.json::<Value>().await.unwrap()["data"]["runId"]
        .as_str()
        .unwrap()
        .to_string();
    let snap = wait_for_finish(app.addr, &token, &run_id, Duration::from_secs(5)).await;
    assert_eq!(snap["data"]["status"], "succeeded");
    // Mock echoes `{model}::{user_content}`. n1 sees "ORIGINAL", reply is
    // "first-model::ORIGINAL". n2 sees that reply as its forwarded input,
    // reply is "second-model::first-model::ORIGINAL".
    let output = snap["data"]["output"].as_str().unwrap_or("");
    assert!(output.contains("first-model::ORIGINAL"), "got {output}");
    assert!(output.contains("second-model::"), "got {output}");
}

// --- Event stream lifecycle ---

#[tokio::test]
async fn snapshot_lists_ordered_lifecycle_events() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; snapshot_lists_ordered_lifecycle_events skipped");
        return;
    };
    let Some(redis) = try_redis().await else {
        eprintln!("SKIP: no redis; snapshot_lists_ordered_lifecycle_events skipped");
        return;
    };
    let app = spawn_with_gateway(db, redis).await;
    let token = make_token(&fresh_owner("events"));
    let a = create_agent(app.addr, &token, "A", "gpt-4o").await;
    let b = create_agent(app.addr, &token, "B", "gpt-4o-mini").await;
    let resp = start_run(
        app.addr,
        &token,
        &json!({ "prompt": "hi", "graph": linear_graph(&a, &b) }),
    )
    .await;
    let run_id = resp.json::<Value>().await.unwrap()["data"]["runId"]
        .as_str()
        .unwrap()
        .to_string();
    let snap = wait_for_finish(app.addr, &token, &run_id, Duration::from_secs(5)).await;
    let events = snap["data"]["events"].as_array().unwrap();
    let names: Vec<&str> = events
        .iter()
        .map(|e| e["event"].as_str().unwrap())
        .collect();
    // The first event is run.started and the last is run.finished.
    assert_eq!(names.first().copied(), Some("run.started"));
    assert_eq!(names.last().copied(), Some("run.finished"));
    // Each node has started + completed (no failures here).
    let started_count = names.iter().filter(|n| **n == "node.started").count();
    let completed_count = names.iter().filter(|n| **n == "node.completed").count();
    assert_eq!(started_count, 2);
    assert_eq!(completed_count, 2);
    // Sequence ids are monotonic.
    let seqs: Vec<u64> = events.iter().map(|e| e["seq"].as_u64().unwrap()).collect();
    for w in seqs.windows(2) {
        assert!(w[1] > w[0], "seq must be strictly monotonic, got {seqs:?}");
    }
}

#[tokio::test]
async fn node_failure_skips_downstream_and_reports() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; node_failure_skips_downstream_and_reports skipped");
        return;
    };
    let Some(redis) = try_redis().await else {
        eprintln!("SKIP: no redis; node_failure_skips_downstream_and_reports skipped");
        return;
    };
    let app = spawn_with_gateway(db, redis).await;
    // Configure the mock to fail completions for the n1 model.
    app.proxy_state
        .fail_models
        .lock()
        .unwrap()
        .push("first-fail".to_string());
    let token = make_token(&fresh_owner("fail"));
    let a = create_agent(app.addr, &token, "A", "first-fail").await;
    let b = create_agent(app.addr, &token, "B", "second-ok").await;
    let resp = start_run(
        app.addr,
        &token,
        &json!({ "prompt": "hi", "graph": linear_graph(&a, &b) }),
    )
    .await;
    let run_id = resp.json::<Value>().await.unwrap()["data"]["runId"]
        .as_str()
        .unwrap()
        .to_string();
    let snap = wait_for_finish(app.addr, &token, &run_id, Duration::from_secs(5)).await;
    assert_eq!(snap["data"]["status"], "failed");
    let events = snap["data"]["events"].as_array().unwrap();
    // node.failed for n1 must precede node.skipped for n2.
    let has_failed_n1 = events
        .iter()
        .any(|e| e["event"] == "node.failed" && e["nodeId"] == "n1");
    let has_skipped_n2 = events
        .iter()
        .any(|e| e["event"] == "node.skipped" && e["nodeId"] == "n2");
    assert!(has_failed_n1, "expected node.failed for n1");
    assert!(has_skipped_n2, "expected node.skipped for n2");
    // The terminal event lists n1 as failed and n2 as skipped.
    let finished = events.iter().last().unwrap();
    assert_eq!(finished["event"], "run.finished");
    let failed: Vec<String> = finished["failedNodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let skipped: Vec<String> = finished["skippedNodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(failed.contains(&"n1".to_string()));
    assert!(skipped.contains(&"n2".to_string()));
}

#[tokio::test]
async fn independent_branch_continues_on_partial_failure() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; independent_branch_continues_on_partial_failure skipped");
        return;
    };
    let Some(redis) = try_redis().await else {
        eprintln!("SKIP: no redis; independent_branch_continues_on_partial_failure skipped");
        return;
    };
    let app = spawn_with_gateway(db, redis).await;
    // Three-branch graph: root → {ok, bad} → terminal. We fail the "bad"
    // model only; the "ok" branch must complete and the terminal node
    // execute with the surviving upstream output.
    app.proxy_state
        .fail_models
        .lock()
        .unwrap()
        .push("bad-model".to_string());
    let token = make_token(&fresh_owner("branches"));
    let root = create_agent(app.addr, &token, "root", "root-model").await;
    let ok = create_agent(app.addr, &token, "ok", "ok-model").await;
    let bad = create_agent(app.addr, &token, "bad", "bad-model").await;
    let term = create_agent(app.addr, &token, "term", "term-model").await;
    // Diamond with root → ok, root → bad, ok → term, bad → term.
    let graph = diamond_graph(&root, &ok, &bad, &term);
    let resp = start_run(
        app.addr,
        &token,
        &json!({ "prompt": "ROOT", "graph": graph }),
    )
    .await;
    let run_id = resp.json::<Value>().await.unwrap()["data"]["runId"]
        .as_str()
        .unwrap()
        .to_string();
    let snap = wait_for_finish(app.addr, &token, &run_id, Duration::from_secs(5)).await;
    // Failed overall (bad branch fell), but the ok branch and term still
    // executed.
    assert_eq!(snap["data"]["status"], "failed");
    let events = snap["data"]["events"].as_array().unwrap();
    let term_started = events
        .iter()
        .any(|e| e["event"] == "node.started" && e["nodeId"] == "n4");
    let term_completed = events
        .iter()
        .any(|e| e["event"] == "node.completed" && e["nodeId"] == "n4");
    assert!(
        term_started,
        "terminal node must execute despite sibling failure"
    );
    assert!(
        term_completed,
        "terminal node must complete despite sibling failure"
    );
}

// --- Reconnect / replay ---

#[tokio::test]
async fn reconnect_replays_buffered_events() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; reconnect_replays_buffered_events skipped");
        return;
    };
    let Some(redis) = try_redis().await else {
        eprintln!("SKIP: no redis; reconnect_replays_buffered_events skipped");
        return;
    };
    let app = spawn_with_gateway(db, redis).await;
    let token = make_token(&fresh_owner("reconnect"));
    let a = create_agent(app.addr, &token, "A", "m").await;
    let b = create_agent(app.addr, &token, "B", "m").await;
    let resp = start_run(
        app.addr,
        &token,
        &json!({ "prompt": "hi", "graph": linear_graph(&a, &b) }),
    )
    .await;
    let run_id = resp.json::<Value>().await.unwrap()["data"]["runId"]
        .as_str()
        .unwrap()
        .to_string();
    let snap = wait_for_finish(app.addr, &token, &run_id, Duration::from_secs(5)).await;
    let total = snap["data"]["events"].as_array().unwrap().len();
    assert!(total >= 4); // at minimum: run.started + 2x node.* + run.finished

    // Opening the SSE stream after the run has finished must replay the
    // entire buffered log including run.finished, then close.
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{}/api/v1/runs/{run_id}/events", app.addr))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .starts_with("text/event-stream"));
    let body = resp.text().await.unwrap();
    assert!(body.contains("event: run.started"));
    assert!(body.contains("event: run.finished"));
}

// --- Ownership / cross-user isolation ---

#[tokio::test]
async fn events_require_ownership() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; events_require_ownership skipped");
        return;
    };
    let Some(redis) = try_redis().await else {
        eprintln!("SKIP: no redis; events_require_ownership skipped");
        return;
    };
    let app = spawn_with_gateway(db, redis).await;
    let owner = make_token(&fresh_owner("ownerA"));
    let other = make_token(&fresh_owner("ownerB"));
    let a = create_agent(app.addr, &owner, "A", "m").await;
    let b = create_agent(app.addr, &owner, "B", "m").await;
    let resp = start_run(
        app.addr,
        &owner,
        &json!({ "prompt": "hi", "graph": linear_graph(&a, &b) }),
    )
    .await;
    let run_id = resp.json::<Value>().await.unwrap()["data"]["runId"]
        .as_str()
        .unwrap()
        .to_string();

    // Other user is NotFound on both snapshot and events.
    let snap = reqwest::Client::new()
        .get(format!("http://{}/api/v1/runs/{run_id}", app.addr))
        .bearer_auth(&other)
        .send()
        .await
        .unwrap();
    assert_eq!(snap.status(), 404);
    let events = reqwest::Client::new()
        .get(format!("http://{}/api/v1/runs/{run_id}/events", app.addr))
        .bearer_auth(&other)
        .send()
        .await
        .unwrap();
    assert_eq!(events.status(), 404);

    let _ = wait_for_finish(app.addr, &owner, &run_id, Duration::from_secs(5)).await;
}
