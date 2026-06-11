//! F04 agents integration tests.
//!
//! Reuses the F03 harness: an in-process axum mock stands in for the LiteLLM
//! proxy's `/model/info` so provider/model validation runs deterministically,
//! tokens are signed with the embedded test key, and DB-dependent tests
//! soft-skip when no `DATABASE_URL` is reachable. Each test uses a freshly
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

/// Mock LiteLLM `/model/info`: two providers, `openai` (gpt-4o) and `anthropic`
/// (claude-3-5-sonnet), matching the F03 gateway harness.
async fn mock_model_info() -> Json<Value> {
    Json(json!({
        "data": [
            { "model_name": "gpt-4o",
              "litellm_params": { "model": "openai/gpt-4o" },
              "model_info": { "mode": "chat", "litellm_provider": "openai" } },
            { "model_name": "claude-3-5-sonnet",
              "litellm_params": { "model": "anthropic/claude-3-5-sonnet" },
              "model_info": { "mode": "chat", "litellm_provider": "anthropic" } }
        ]
    }))
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
    auth.jwks
        .insert_key(TEST_KID, DecodingKey::from_rsa_pem(PUB_PEM).expect("decoding key"));

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

/// Spawn the full app wired to a fresh mock gateway. Returns its base URL.
async fn spawn_with_gateway(db: PgPool) -> SocketAddr {
    let proxy = spawn_router(mock_proxy()).await;
    spawn_app(build_state(db, format!("http://{proxy}"))).await
}

/// A fresh, never-before-seen owner id so each run starts with no rows.
fn fresh_owner(tag: &str) -> String {
    format!("user_agt_{tag}_{}", nonce())
}

/// A valid create/update body with all seven fields.
fn full_body(name: &str) -> Value {
    json!({
        "name": name,
        "preamble": "You are concise.",
        "system_prompt": "Summarize the user input in three bullet points.",
        "provider": "openai",
        "model": "gpt-4o",
        "recent_n": 10,
        "top_k": 5
    })
}

async fn post_agent(addr: SocketAddr, token: &str, body: &Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("http://{addr}/api/v1/agents"))
        .bearer_auth(token)
        .json(body)
        .send()
        .await
        .unwrap()
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
        .post(format!("http://{addr}/api/v1/agents"))
        .json(&full_body("Summarizer"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "AUTH001"
    );
}

// --- Create ---

#[tokio::test]
async fn create_agent_success() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_agent_success skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("success"));

    let resp = post_agent(addr, &token, &full_body("Summarizer")).await;
    assert_eq!(resp.status(), 201);
    let agent = &resp.json::<Value>().await.unwrap()["data"];
    assert!(agent["id"].as_str().is_some());
    assert_eq!(agent["name"], "Summarizer");
    assert_eq!(agent["provider"], "openai");
    assert_eq!(agent["model"], "gpt-4o");
    assert_eq!(agent["recent_n"], 10);
    assert_eq!(agent["top_k"], 5);
    assert!(agent["created_at"].as_str().is_some());
    assert!(agent["updated_at"].as_str().is_some());
}

#[tokio::test]
async fn create_applies_defaults() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_applies_defaults skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("defaults"));

    let body = json!({
        "name": "Defaulted",
        "system_prompt": "Do the thing.",
        "provider": "openai",
        "model": "gpt-4o"
    });
    let resp = post_agent(addr, &token, &body).await;
    assert_eq!(resp.status(), 201);
    let agent = &resp.json::<Value>().await.unwrap()["data"];
    assert_eq!(agent["recent_n"], 10);
    assert_eq!(agent["top_k"], 5);
    assert!(agent["preamble"].is_null());
}

#[tokio::test]
async fn create_empty_name_rejected() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_empty_name_rejected skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("emptyname"));

    let resp = post_agent(addr, &token, &full_body("   ")).await;
    assert_eq!(resp.status(), 422);
    let err = resp.json::<Value>().await.unwrap();
    assert_eq!(err["error"]["code"], "AGENT_VALIDATION");
    assert!(err["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Name"));
}

#[tokio::test]
async fn create_long_name_rejected() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_long_name_rejected skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("longname"));

    let resp = post_agent(addr, &token, &full_body(&"a".repeat(65))).await;
    assert_eq!(resp.status(), 422);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "AGENT_VALIDATION"
    );
}

#[tokio::test]
async fn create_out_of_range_recent_n() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_out_of_range_recent_n skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("recentn"));

    let mut body = full_body("RecentN");
    body["recent_n"] = json!(200);
    let resp = post_agent(addr, &token, &body).await;
    assert_eq!(resp.status(), 422);
    let err = resp.json::<Value>().await.unwrap();
    assert_eq!(err["error"]["code"], "AGENT_VALIDATION");
    assert_eq!(err["error"]["message"], "Value must be between 0 and 100.");
}

#[tokio::test]
async fn create_out_of_range_top_k() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_out_of_range_top_k skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("topk"));

    let mut body = full_body("TopK");
    body["top_k"] = json!(99);
    let resp = post_agent(addr, &token, &body).await;
    assert_eq!(resp.status(), 422);
    let err = resp.json::<Value>().await.unwrap();
    assert_eq!(err["error"]["code"], "AGENT_VALIDATION");
    assert_eq!(err["error"]["message"], "Value must be between 0 and 50.");
}

#[tokio::test]
async fn create_invalid_model_for_provider() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_invalid_model_for_provider skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("badmodel"));

    let mut body = full_body("BadModel");
    body["model"] = json!("gpt-does-not-exist");
    let resp = post_agent(addr, &token, &body).await;
    assert_eq!(resp.status(), 422);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "GW002"
    );
}

#[tokio::test]
async fn create_when_catalog_down() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; create_when_catalog_down skipped");
        return;
    };
    // Port 1 is refused: the gateway catalog call fails fast.
    let addr = spawn_app(build_state(db, "http://127.0.0.1:1".to_string())).await;
    let token = make_token(&fresh_owner("catalogdown"));

    let resp = post_agent(addr, &token, &full_body("CatalogDown")).await;
    assert_eq!(resp.status(), 503);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "GW001"
    );
}

#[tokio::test]
async fn duplicate_name_rejected() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; duplicate_name_rejected skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("dup"));

    let first = post_agent(addr, &token, &full_body("Summarizer")).await;
    assert_eq!(first.status(), 201);

    // Same name, different case → still a conflict (case-insensitive).
    let second = post_agent(addr, &token, &full_body("SUMMARIZER")).await;
    assert_eq!(second.status(), 409);
    assert_eq!(
        second.json::<Value>().await.unwrap()["error"]["code"],
        "AGENT_NAME_TAKEN"
    );
}

// --- List / ownership ---

#[tokio::test]
async fn list_returns_only_owner_agents() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; list_returns_only_owner_agents skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let owner = make_token(&fresh_owner("listowner"));
    let other = make_token(&fresh_owner("listother"));

    // Owner creates two (out of alphabetical order); the other user creates one.
    assert_eq!(post_agent(addr, &owner, &full_body("Zeta")).await.status(), 201);
    assert_eq!(post_agent(addr, &owner, &full_body("Alpha")).await.status(), 201);
    assert_eq!(post_agent(addr, &other, &full_body("Foreign")).await.status(), 201);

    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/agents"))
        .bearer_auth(&owner)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.json::<Value>().await.unwrap();
    let agents = body["data"]["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 2);
    assert_eq!(agents[0]["name"], "Alpha");
    assert_eq!(agents[1]["name"], "Zeta");
}

#[tokio::test]
async fn get_other_users_agent_returns_404() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; get_other_users_agent_returns_404 skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let owner = make_token(&fresh_owner("getowner"));
    let other = make_token(&fresh_owner("getother"));

    let created = post_agent(addr, &owner, &full_body("Private")).await;
    let id = created.json::<Value>().await.unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/agents/{id}"))
        .bearer_auth(&other)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "NOT_FOUND"
    );
}

// --- Update ---

async fn create_and_get_id(addr: SocketAddr, token: &str, name: &str) -> String {
    let resp = post_agent(addr, token, &full_body(name)).await;
    assert_eq!(resp.status(), 201);
    resp.json::<Value>().await.unwrap()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn update_agent_success() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; update_agent_success skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("update"));
    let id = create_and_get_id(addr, &token, "Original").await;

    let mut body = full_body("Renamed");
    body["recent_n"] = json!(25);
    let resp = reqwest::Client::new()
        .put(format!("http://{addr}/api/v1/agents/{id}"))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let agent = resp.json::<Value>().await.unwrap();
    let agent = &agent["data"];
    assert_eq!(agent["name"], "Renamed");
    assert_eq!(agent["recent_n"], 25);
    // RFC3339 timestamps sort lexicographically; update advances updated_at.
    assert!(agent["updated_at"].as_str().unwrap() >= agent["created_at"].as_str().unwrap());
}

#[tokio::test]
async fn update_keeps_same_name_ok() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; update_keeps_same_name_ok skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("samename"));
    let id = create_and_get_id(addr, &token, "Keeper").await;

    // Same name, changed prompt — self is excluded from the uniqueness check.
    let mut body = full_body("Keeper");
    body["system_prompt"] = json!("A different prompt.");
    let resp = reqwest::Client::new()
        .put(format!("http://{addr}/api/v1/agents/{id}"))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["data"]["system_prompt"],
        "A different prompt."
    );
}

#[tokio::test]
async fn update_other_users_agent_404() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; update_other_users_agent_404 skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let owner = make_token(&fresh_owner("updowner"));
    let other = make_token(&fresh_owner("updother"));
    let id = create_and_get_id(addr, &owner, "Owned").await;

    let resp = reqwest::Client::new()
        .put(format!("http://{addr}/api/v1/agents/{id}"))
        .bearer_auth(&other)
        .json(&full_body("Hijacked"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// --- Delete ---

#[tokio::test]
async fn delete_agent_success() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; delete_agent_success skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let token = make_token(&fresh_owner("delete"));
    let id = create_and_get_id(addr, &token, "Doomed").await;

    let resp = reqwest::Client::new()
        .delete(format!("http://{addr}/api/v1/agents/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.json::<Value>().await.unwrap()["data"]["id"], id);

    // Gone from the list.
    let list = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/agents"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let body = list.json::<Value>().await.unwrap();
    assert_eq!(body["data"]["agents"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn delete_other_users_agent_404() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; delete_other_users_agent_404 skipped");
        return;
    };
    let addr = spawn_with_gateway(db).await;
    let owner = make_token(&fresh_owner("delowner"));
    let other = make_token(&fresh_owner("delother"));
    let id = create_and_get_id(addr, &owner, "Protected").await;

    let resp = reqwest::Client::new()
        .delete(format!("http://{addr}/api/v1/agents/{id}"))
        .bearer_auth(&other)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
