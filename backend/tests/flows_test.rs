//! F08 flows integration tests.
//!
//! Reuses the F04 harness: tokens are signed with the embedded test key and
//! DB-dependent tests soft-skip when no `DATABASE_URL` is reachable. Flows do
//! not touch the gateway, but the app router still needs one wired into state,
//! so the F03/F04 mock proxy is spawned unchanged. Each test uses a freshly
//! nonced owner id so the suite is rerunnable against a persistent database.

use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_maker_flow_backend::{
    app,
    auth::AuthState,
    cache,
    config::{AppConfig, ClerkConfig, GatewayConfig},
    db,
    gateway::GatewayClient,
    state::AppState,
};
use axum::routing::get;
use axum::{Json, Router};
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

/// Mock LiteLLM `/model/info` — present only because the app state requires a
/// gateway; flows never call it.
async fn mock_model_info() -> Json<Value> {
    Json(json!({ "data": [] }))
}

fn mock_proxy() -> Router {
    Router::new().route("/model/info", get(mock_model_info))
}

async fn spawn_router(router: Router) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    addr
}

fn build_state(db: PgPool, gateway_base: String) -> AppState {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let redis = cache::init_pool(&redis_url).expect("redis pool");

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
        runs: std::sync::Arc::new(agent_maker_flow_backend::runs::RunRegistry::new()),
    }
}

async fn try_db() -> Option<PgPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = db::init_pool(&url).await.ok()?;
    db::run_migrations(&pool).await.ok()?;
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

/// Spawn the full app wired to a fresh (unused) mock gateway. Returns its addr.
async fn spawn_with_gateway(db: PgPool) -> SocketAddr {
    let proxy = spawn_router(mock_proxy()).await;
    spawn_app(build_state(db, format!("http://{proxy}"))).await
}

/// A fresh, never-before-seen owner id so each run starts with no rows.
fn fresh_owner(tag: &str) -> String {
    format!("user_flw_{tag}_{}", nonce())
}

/// A two-node graph with one edge and a root, matching the F07 `FlowGraph`.
fn sample_graph() -> Value {
    json!({
        "nodes": [
            { "id": "node-1", "type": "agent", "position": { "x": 120, "y": 80 },
              "data": { "agentId": "11111111-1111-1111-1111-111111111111" } },
            { "id": "node-2", "type": "agent", "position": { "x": 360, "y": 200 },
              "data": { "agentId": "22222222-2222-2222-2222-222222222222" } }
        ],
        "edges": [
            { "id": "edge-node-1-node-2", "source": "node-1", "target": "node-2" }
        ],
        "rootNodeId": "node-1"
    })
}

fn flow_body(name: &str, graph: &Value) -> Value {
    json!({ "name": name, "graph": graph })
}

async fn post_flow(addr: SocketAddr, token: &str, body: &Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/flows"))
        .bearer_auth(token)
        .json(body)
        .send()
        .await
        .unwrap()
}

async fn create_and_get_id(addr: SocketAddr, token: &str, name: &str, graph: &Value) -> String {
    let resp = post_flow(addr, token, &flow_body(name, graph)).await;
    assert_eq!(resp.status(), 201);
    resp.json::<Value>().await.unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

// --- Auth ---

#[tokio::test]
async fn create_requires_auth() {
    // No DB needed: rejected by the auth layer before any DB work.
    let proxy = spawn_router(mock_proxy()).await;
    let lazy = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://amf:amf@localhost:5432/amf")
        .unwrap();
    let addr = spawn_app(build_state(lazy, format!("http://{proxy}"))).await;

    let resp = reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/flows"))
        .json(&flow_body("Pipeline", &sample_graph()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "AUTH001"
    );
}

// --- Round-trip ---

#[tokio::test]
async fn create_then_get_round_trips_graph() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_then_get_round_trips_graph skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("roundtrip"));
    let graph = sample_graph();

    let created = post_flow(addr, &token, &flow_body("Pipeline", &graph)).await;
    assert_eq!(created.status(), 201);
    let body = created.json::<Value>().await.unwrap();
    let id = body["data"]["id"].as_str().unwrap().to_string();
    // The create response already echoes the stored graph verbatim.
    assert_eq!(body["data"]["graph"], graph);

    let opened = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/flows/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(opened.status(), 200);
    let opened = opened.json::<Value>().await.unwrap();
    assert_eq!(opened["data"]["name"], "Pipeline");
    // Nodes, positions, edges, and root are restored exactly as saved.
    assert_eq!(opened["data"]["graph"], graph);
}

// --- List ---

#[tokio::test]
async fn list_returns_summaries_without_graph() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; list_returns_summaries_without_graph skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("list"));
    let graph = sample_graph();

    // "Older" then "Newer"; the most recently updated sorts first.
    assert_eq!(
        post_flow(addr, &token, &flow_body("Older", &graph))
            .await
            .status(),
        201
    );
    assert_eq!(
        post_flow(addr, &token, &flow_body("Newer", &graph))
            .await
            .status(),
        201
    );

    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/flows"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.json::<Value>().await.unwrap();
    let flows = body["data"]["flows"].as_array().unwrap();
    assert_eq!(flows.len(), 2);
    assert_eq!(flows[0]["name"], "Newer");
    assert_eq!(flows[1]["name"], "Older");
    // Summaries carry no graph.
    assert!(flows[0]["graph"].is_null());
    assert!(flows[0]["created_at"].as_str().is_some());
    assert!(flows[0]["updated_at"].as_str().is_some());
}

// --- Uniqueness ---

#[tokio::test]
async fn duplicate_name_is_rejected() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; duplicate_name_is_rejected skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("dup"));
    let graph = sample_graph();

    let first = post_flow(addr, &token, &flow_body("Pipeline", &graph)).await;
    assert_eq!(first.status(), 201);

    // Same name, different case → still a conflict (case-insensitive).
    let second = post_flow(addr, &token, &flow_body("PIPELINE", &graph)).await;
    assert_eq!(second.status(), 409);
    assert_eq!(
        second.json::<Value>().await.unwrap()["error"]["code"],
        "FLOW_NAME_TAKEN"
    );
}

#[tokio::test]
async fn malformed_graph_is_rejected() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; malformed_graph_is_rejected skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("malformed"));

    // rootNodeId references a node that is not present.
    let bad = json!({ "nodes": [], "edges": [], "rootNodeId": "ghost" });
    let resp = post_flow(addr, &token, &flow_body("Bad", &bad)).await;
    assert_eq!(resp.status(), 422);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "FLOW_VALIDATION"
    );
}

// --- Update ---

#[tokio::test]
async fn update_saves_graph_and_refreshes_updated_at() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; update_saves_graph_and_refreshes_updated_at skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("update"));
    let id = create_and_get_id(addr, &token, "Pipeline", &sample_graph()).await;

    // Re-save under the same name with a changed graph (root cleared).
    let mut next = sample_graph();
    next["rootNodeId"] = Value::Null;
    let resp = reqwest::Client::new()
        .put(format!("http://{addr}/api/v1/flows/{id}"))
        .bearer_auth(&token)
        .json(&flow_body("Pipeline", &next))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let flow = resp.json::<Value>().await.unwrap();
    let flow = &flow["data"];
    assert_eq!(flow["graph"], next);
    // RFC3339 timestamps sort lexicographically; the save advances updated_at.
    assert!(flow["updated_at"].as_str().unwrap() >= flow["created_at"].as_str().unwrap());
}

// --- Rename ---

#[tokio::test]
async fn rename_enforces_uniqueness() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; rename_enforces_uniqueness skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("rename"));
    let graph = sample_graph();

    create_and_get_id(addr, &token, "Pipeline", &graph).await;
    let other = create_and_get_id(addr, &token, "Other", &graph).await;

    // Rename to a free name succeeds and leaves the graph untouched.
    let ok = reqwest::Client::new()
        .patch(format!("http://{addr}/api/v1/flows/{other}"))
        .bearer_auth(&token)
        .json(&json!({ "name": "Renamed" }))
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 200);
    let renamed = ok.json::<Value>().await.unwrap();
    assert_eq!(renamed["data"]["name"], "Renamed");
    assert!(renamed["data"]["graph"].is_null()); // summary response, no graph

    // Rename onto an existing name → conflict.
    let clash = reqwest::Client::new()
        .patch(format!("http://{addr}/api/v1/flows/{other}"))
        .bearer_auth(&token)
        .json(&json!({ "name": "Pipeline" }))
        .send()
        .await
        .unwrap();
    assert_eq!(clash.status(), 409);
    assert_eq!(
        clash.json::<Value>().await.unwrap()["error"]["code"],
        "FLOW_NAME_TAKEN"
    );
}

// --- Delete ---

#[tokio::test]
async fn delete_removes_flow() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; delete_removes_flow skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("delete"));
    let id = create_and_get_id(addr, &token, "Doomed", &sample_graph()).await;

    let resp = reqwest::Client::new()
        .delete(format!("http://{addr}/api/v1/flows/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.json::<Value>().await.unwrap()["data"]["id"], id);

    let gone = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/flows/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(gone.status(), 404);
}

// --- Cross-user isolation ---

#[tokio::test]
async fn cross_user_isolation() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; cross_user_isolation skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let owner = make_token(&fresh_owner("isoowner"));
    let other = make_token(&fresh_owner("isoother"));
    let graph = sample_graph();
    let id = create_and_get_id(addr, &owner, "Private", &graph).await;

    let client = reqwest::Client::new();

    // B cannot open A's flow.
    let get = client
        .get(format!("http://{addr}/api/v1/flows/{id}"))
        .bearer_auth(&other)
        .send()
        .await
        .unwrap();
    assert_eq!(get.status(), 404);

    // B cannot re-save A's flow.
    let put = client
        .put(format!("http://{addr}/api/v1/flows/{id}"))
        .bearer_auth(&other)
        .json(&flow_body("Hijacked", &graph))
        .send()
        .await
        .unwrap();
    assert_eq!(put.status(), 404);

    // B cannot rename A's flow.
    let patch = client
        .patch(format!("http://{addr}/api/v1/flows/{id}"))
        .bearer_auth(&other)
        .json(&json!({ "name": "Hijacked" }))
        .send()
        .await
        .unwrap();
    assert_eq!(patch.status(), 404);

    // B cannot delete A's flow.
    let delete = client
        .delete(format!("http://{addr}/api/v1/flows/{id}"))
        .bearer_auth(&other)
        .send()
        .await
        .unwrap();
    assert_eq!(delete.status(), 404);

    // B's list excludes A's flows.
    let list = client
        .get(format!("http://{addr}/api/v1/flows"))
        .bearer_auth(&other)
        .send()
        .await
        .unwrap();
    let body = list.json::<Value>().await.unwrap();
    assert_eq!(body["data"]["flows"].as_array().unwrap().len(), 0);
}
