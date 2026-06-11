//! F05 embedding-settings & memory integration tests.
//!
//! Reuses the F03/F04 harness: an in-process axum mock stands in for the
//! LiteLLM proxy's `/model/info` and `/embeddings`, tokens are signed with the
//! embedded RS256 test key, and DB/Redis-dependent tests soft-skip when neither
//! is reachable. Each test uses a freshly nonced owner so the suite is
//! rerunnable against a persistent database.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
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
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::PgPool;

const TEST_KID: &str = "amf-test-key-1";
const ISSUER: &str = "https://amf.test";
const AZP: &str = "http://localhost:5173";
const EMBED_MODEL: &str = "text-embedding-3-small";
const CHAT_MODEL: &str = "gpt-4o";

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

fn fresh_user(tag: &str) -> String {
    format!("user_mem_{tag}_{}", nonce())
}

// --- Mock LiteLLM proxy: /model/info + /embeddings ---

#[derive(Clone)]
struct ProxyState {
    embed_calls: Arc<AtomicUsize>,
    fail_embed: bool,
}

/// One embedding model + one chat model, both under `openai`.
async fn mock_model_info() -> Json<Value> {
    Json(json!({
        "data": [
            { "model_name": EMBED_MODEL,
              "litellm_params": { "model": format!("openai/{EMBED_MODEL}") },
              "model_info": { "mode": "embedding", "litellm_provider": "openai" } },
            { "model_name": CHAT_MODEL,
              "litellm_params": { "model": format!("openai/{CHAT_MODEL}") },
              "model_info": { "mode": "chat", "litellm_provider": "openai" } }
        ]
    }))
}

async fn mock_embed(
    State(s): State<ProxyState>,
    Json(_body): Json<Value>,
) -> axum::response::Response {
    s.embed_calls.fetch_add(1, Ordering::SeqCst);
    if s.fail_embed {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": { "message": "embedding backend exploded" } })),
        )
            .into_response();
    }
    let resp = json!({
        "model": EMBED_MODEL,
        "data": [{ "embedding": [0.1, 0.2, 0.3], "index": 0 }],
        "usage": { "prompt_tokens": 4, "total_tokens": 4 }
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

/// Spawn the app wired to a fresh mock proxy. Returns the app addr, the proxy's
/// embed-call counter, and a clone of the DB pool for direct assertions.
async fn spawn_with_proxy(db: PgPool, fail_embed: bool) -> (SocketAddr, Arc<AtomicUsize>) {
    let (router, calls) = mock_proxy(fail_embed);
    let proxy = spawn_router(router).await;
    let addr = spawn_app(build_state(db, format!("http://{proxy}"))).await;
    (addr, calls)
}

// --- HTTP helpers ---

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn set_global_model(addr: SocketAddr, token: &str, model: &str) -> reqwest::Response {
    client()
        .put(format!("http://{addr}/api/v1/settings/embedding"))
        .bearer_auth(token)
        .json(&json!({ "embedding_model": model }))
        .send()
        .await
        .unwrap()
}

async fn create_memory(addr: SocketAddr, token: &str, text: &str) -> reqwest::Response {
    client()
        .post(format!("http://{addr}/api/v1/memory"))
        .bearer_auth(token)
        .json(&json!({ "text": text }))
        .send()
        .await
        .unwrap()
}

async fn list_memory(addr: SocketAddr, token: &str) -> Value {
    client()
        .get(format!("http://{addr}/api/v1/memory"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()
}

async fn create_agent(addr: SocketAddr, token: &str) -> String {
    let resp = client()
        .post(format!("http://{addr}/api/v1/agents"))
        .bearer_auth(token)
        .json(&json!({
            "name": format!("Agent {}", nonce()),
            "system_prompt": "Do the thing.",
            "provider": "openai",
            "model": CHAT_MODEL
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    resp.json::<Value>().await.unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

// --- Tests ---

#[tokio::test]
async fn settings_endpoints_require_auth() {
    // No DB needed: rejected by the auth layer before any DB work.
    let (router, _) = mock_proxy(false);
    let proxy = spawn_router(router).await;
    let lazy = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://amf:amf@localhost:5432/amf")
        .unwrap();
    let addr = spawn_app(build_state(lazy, format!("http://{proxy}"))).await;

    let resp = client()
        .get(format!("http://{addr}/api/v1/settings/embedding"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "AUTH001"
    );
}

#[tokio::test]
async fn set_and_get_global_embedding_model() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; set_and_get_global_embedding_model skipped");
        return;
    };
    let (addr, _) = spawn_with_proxy(db, false).await;
    let token = make_token(&fresh_user("setget"));

    let put = set_global_model(addr, &token, EMBED_MODEL).await;
    assert_eq!(put.status(), 200);
    assert_eq!(
        put.json::<Value>().await.unwrap()["data"]["embedding_model"],
        EMBED_MODEL
    );

    let get = client()
        .get(format!("http://{addr}/api/v1/settings/embedding"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        get.json::<Value>().await.unwrap()["data"]["embedding_model"],
        EMBED_MODEL
    );
}

#[tokio::test]
async fn set_global_model_rejects_non_embedding_model() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; set_global_model_rejects_non_embedding_model skipped");
        return;
    };
    let (addr, _) = spawn_with_proxy(db, false).await;
    let token = make_token(&fresh_user("reject"));

    let resp = set_global_model(addr, &token, CHAT_MODEL).await;
    assert_eq!(resp.status(), 422);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "GW002"
    );
}

#[tokio::test]
async fn create_memory_record_embeds_and_stores() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_memory_record_embeds_and_stores skipped");
        return;
    };
    let (addr, calls) = spawn_with_proxy(db, false).await;
    let token = make_token(&fresh_user("create"));
    assert_eq!(
        set_global_model(addr, &token, EMBED_MODEL).await.status(),
        200
    );

    let resp = create_memory(addr, &token, "Our return policy allows 30 days.").await;
    assert_eq!(resp.status(), 201);
    let record = resp.json::<Value>().await.unwrap();
    let record = &record["data"];
    assert!(record["id"].as_str().is_some());
    assert_eq!(record["embedding_model"], EMBED_MODEL);
    assert_eq!(record["char_count"], 33);
    assert!(record.get("embedding").is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn stored_records_retain_vector_and_model_for_f06() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; stored_records_retain_vector_and_model_for_f06 skipped");
        return;
    };
    let pool = db.clone();
    let (addr, _) = spawn_with_proxy(db, false).await;
    let user = fresh_user("f06");
    let token = make_token(&user);
    assert_eq!(
        set_global_model(addr, &token, EMBED_MODEL).await.status(),
        200
    );
    assert_eq!(create_memory(addr, &token, "vector me").await.status(), 201);

    // F06 filters comparable vectors by (user_id, embedding_model); the row must
    // expose a non-null embedding and the model used.
    let row: (i64,) = sqlx::query_as(
        "SELECT count(*) FROM memory_records \
         WHERE user_id = $1 AND embedding_model = $2 AND embedding IS NOT NULL",
    )
    .bind(&user)
    .bind(EMBED_MODEL)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, 1);
}

#[tokio::test]
async fn create_memory_record_too_large_rejected() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_memory_record_too_large_rejected skipped");
        return;
    };
    let (addr, _) = spawn_with_proxy(db, false).await;
    let token = make_token(&fresh_user("toolarge"));
    assert_eq!(
        set_global_model(addr, &token, EMBED_MODEL).await.status(),
        200
    );

    let resp = create_memory(addr, &token, &"a".repeat(8001)).await;
    assert_eq!(resp.status(), 422);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "MEM001"
    );

    let body = list_memory(addr, &token).await;
    assert_eq!(body["data"]["records"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_memory_without_global_model_rejected() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_memory_without_global_model_rejected skipped");
        return;
    };
    let (addr, _) = spawn_with_proxy(db, false).await;
    let token = make_token(&fresh_user("nomodel"));

    let resp = create_memory(addr, &token, "no model set yet").await;
    assert_eq!(resp.status(), 409);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "MEM002"
    );
}

#[tokio::test]
async fn create_memory_record_embedding_failure_not_stored() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_memory_record_embedding_failure_not_stored skipped");
        return;
    };
    let (addr, _) = spawn_with_proxy(db, true).await; // embeddings return 500
    let token = make_token(&fresh_user("embedfail"));
    assert_eq!(
        set_global_model(addr, &token, EMBED_MODEL).await.status(),
        200
    );

    let resp = create_memory(addr, &token, "this should not persist").await;
    assert_eq!(resp.status(), 502);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "GW003"
    );

    let body = list_memory(addr, &token).await;
    assert_eq!(body["data"]["records"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_memory_returns_only_owner_records() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; list_memory_returns_only_owner_records skipped");
        return;
    };
    let (addr, _) = spawn_with_proxy(db, false).await;
    let owner = make_token(&fresh_user("listowner"));
    let other = make_token(&fresh_user("listother"));
    assert_eq!(
        set_global_model(addr, &owner, EMBED_MODEL).await.status(),
        200
    );
    assert_eq!(
        set_global_model(addr, &other, EMBED_MODEL).await.status(),
        200
    );
    assert_eq!(
        create_memory(addr, &owner, "owner record").await.status(),
        201
    );
    assert_eq!(
        create_memory(addr, &other, "other record").await.status(),
        201
    );

    let body = list_memory(addr, &owner).await;
    let records = body["data"]["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["text"], "owner record");
    assert_eq!(body["data"]["models_in_use"], json!([EMBED_MODEL]));
}

#[tokio::test]
async fn update_memory_record_reembeds() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; update_memory_record_reembeds skipped");
        return;
    };
    let (addr, calls) = spawn_with_proxy(db, false).await;
    let token = make_token(&fresh_user("update"));
    assert_eq!(
        set_global_model(addr, &token, EMBED_MODEL).await.status(),
        200
    );
    let created = create_memory(addr, &token, "original text").await;
    let id = created.json::<Value>().await.unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let resp = client()
        .put(format!("http://{addr}/api/v1/memory/{id}"))
        .bearer_auth(&token)
        .json(&json!({ "text": "updated text" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let record = resp.json::<Value>().await.unwrap();
    assert_eq!(record["data"]["text"], "updated text");
    assert_eq!(record["data"]["embedding_model"], EMBED_MODEL);
    // Re-embedded on update.
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn delete_memory_record() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; delete_memory_record skipped");
        return;
    };
    let (addr, _) = spawn_with_proxy(db, false).await;
    let owner = make_token(&fresh_user("delowner"));
    let other = make_token(&fresh_user("delother"));
    assert_eq!(
        set_global_model(addr, &owner, EMBED_MODEL).await.status(),
        200
    );
    let created = create_memory(addr, &owner, "to be deleted").await;
    let id = created.json::<Value>().await.unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Another user cannot delete it.
    let foreign = client()
        .delete(format!("http://{addr}/api/v1/memory/{id}"))
        .bearer_auth(&other)
        .send()
        .await
        .unwrap();
    assert_eq!(foreign.status(), 404);

    // The owner can.
    let owned = client()
        .delete(format!("http://{addr}/api/v1/memory/{id}"))
        .bearer_auth(&owner)
        .send()
        .await
        .unwrap();
    assert_eq!(owned.status(), 200);
    assert_eq!(owned.json::<Value>().await.unwrap()["data"]["id"], id);
}

#[tokio::test]
async fn set_semantic_profile_overrides_model() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; set_semantic_profile_overrides_model skipped");
        return;
    };
    let (addr, _) = spawn_with_proxy(db, false).await;
    let token = make_token(&fresh_user("profile"));
    let agent_id = create_agent(addr, &token).await;

    let resp = client()
        .put(format!(
            "http://{addr}/api/v1/agents/{agent_id}/semantic-profile"
        ))
        .bearer_auth(&token)
        .json(&json!({ "embedding_model": EMBED_MODEL, "memory_scope": "own" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let profile = resp.json::<Value>().await.unwrap();
    assert_eq!(profile["data"]["agent_id"], agent_id);
    assert_eq!(profile["data"]["embedding_model"], EMBED_MODEL);
    assert_eq!(profile["data"]["memory_scope"], "own");

    // Read back.
    let get = client()
        .get(format!(
            "http://{addr}/api/v1/agents/{agent_id}/semantic-profile"
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(get.status(), 200);
    assert_eq!(
        get.json::<Value>().await.unwrap()["data"]["memory_scope"],
        "own"
    );
}
