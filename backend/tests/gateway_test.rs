//! F03 gateway integration tests.
//!
//! An in-process axum mock stands in for the LiteLLM proxy, so discovery and
//! (in later stages) completion/embedding run deterministically without a real
//! gateway. The discovery endpoints are protected by F02 `require_auth`, so
//! authenticated tests need a database for the JIT user upsert and soft-skip
//! when none is reachable (consistent with health/auth tests).

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
    gateway::types::{ChatMessage, CompletionRequest, EmbeddingRequest},
    gateway::GatewayClient,
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

/// Mock LiteLLM `/model/info`: three models across two providers, including one
/// embedding model.
async fn mock_model_info() -> Json<Value> {
    Json(json!({
        "data": [
            { "model_name": "gpt-4o",
              "litellm_params": { "model": "openai/gpt-4o" },
              "model_info": { "mode": "chat", "litellm_provider": "openai" } },
            { "model_name": "text-embedding-3-small",
              "litellm_params": { "model": "openai/text-embedding-3-small" },
              "model_info": { "mode": "embedding", "litellm_provider": "openai" } },
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

#[tokio::test]
async fn providers_endpoint_requires_auth() {
    // No DB needed: rejected by the auth layer before any upsert.
    let proxy = spawn_router(mock_proxy()).await;
    let lazy = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://amf:amf@localhost:5432/amf")
        .unwrap();
    let addr = spawn_app(build_state(lazy, format!("http://{proxy}"))).await;

    let resp = reqwest::get(format!("http://{addr}/api/v1/providers"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "AUTH001"
    );
}

#[tokio::test]
async fn lists_providers_grouped_from_model_info() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; lists_providers_grouped_from_model_info skipped");
        return;
    };
    let proxy = spawn_router(mock_proxy()).await;
    let addr = spawn_app(build_state(db, format!("http://{proxy}"))).await;

    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/providers"))
        .bearer_auth(make_token("user_gw_1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let providers = body["data"]["providers"].as_array().unwrap();
    // openai (2 models) + anthropic (1 model)
    let openai = providers.iter().find(|p| p["id"] == "openai").unwrap();
    assert_eq!(openai["model_count"], 2);
    let anthropic = providers.iter().find(|p| p["id"] == "anthropic").unwrap();
    assert_eq!(anthropic["model_count"], 1);
}

#[tokio::test]
async fn lists_models_for_provider_with_mode() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; lists_models_for_provider_with_mode skipped");
        return;
    };
    let proxy = spawn_router(mock_proxy()).await;
    let addr = spawn_app(build_state(db, format!("http://{proxy}"))).await;

    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/providers/openai/models"))
        .bearer_auth(make_token("user_gw_2"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["provider"], "openai");
    let models = body["data"]["models"].as_array().unwrap();
    assert_eq!(models.len(), 2);
    assert!(models
        .iter()
        .any(|m| m["id"] == "text-embedding-3-small" && m["mode"] == "embedding"));
    assert!(models.iter().any(|m| m["mode"] == "chat"));
}

#[tokio::test]
async fn unknown_provider_returns_404() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; unknown_provider_returns_404 skipped");
        return;
    };
    let proxy = spawn_router(mock_proxy()).await;
    let addr = spawn_app(build_state(db, format!("http://{proxy}"))).await;

    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/providers/nope/models"))
        .bearer_auth(make_token("user_gw_3"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "NOT_FOUND"
    );
}

#[tokio::test]
async fn discovery_when_proxy_down_returns_gw001() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database; discovery_when_proxy_down_returns_gw001 skipped");
        return;
    };
    // Port 1 is refused: the gateway call fails fast.
    let addr = spawn_app(build_state(db, "http://127.0.0.1:1".to_string())).await;

    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/providers"))
        .bearer_auth(make_token("user_gw_4"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 503);
    assert_eq!(
        resp.json::<Value>().await.unwrap()["error"]["code"],
        "GW001"
    );
}

// --- Completion / embedding (internal client, called directly) ---

#[derive(Clone)]
struct ExecState {
    completion_calls: Arc<AtomicUsize>,
    embedding_calls: Arc<AtomicUsize>,
}

async fn mock_chat(State(s): State<ExecState>, Json(body): Json<Value>) -> axum::response::Response {
    let model = body["model"].as_str().unwrap_or("unknown").to_string();
    if model == "invalid-model" {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({ "error": { "message": "model not available for provider" } })),
        )
            .into_response();
    }
    s.completion_calls.fetch_add(1, Ordering::SeqCst);
    let resp = json!({
        "id": "cmpl-mock",
        "model": model,
        "choices": [{ "message": { "role": "assistant", "content": "hello from mock" } }],
        "usage": { "prompt_tokens": 5, "completion_tokens": 3, "total_tokens": 8 }
    });
    ([("x-litellm-response-cost", "0.0001")], Json(resp)).into_response()
}

async fn mock_embed(State(s): State<ExecState>, Json(_body): Json<Value>) -> axum::response::Response {
    s.embedding_calls.fetch_add(1, Ordering::SeqCst);
    let resp = json!({
        "model": "text-embedding-3-small",
        "data": [{ "embedding": [0.1, 0.2, 0.3], "index": 0 }],
        "usage": { "prompt_tokens": 4, "total_tokens": 4 }
    });
    ([("x-litellm-response-cost", "0.00002")], Json(resp)).into_response()
}

fn mock_exec_proxy() -> (Router, ExecState) {
    let state = ExecState {
        completion_calls: Arc::new(AtomicUsize::new(0)),
        embedding_calls: Arc::new(AtomicUsize::new(0)),
    };
    let router = Router::new()
        .route("/chat/completions", post(mock_chat))
        .route("/embeddings", post(mock_embed))
        .with_state(state.clone());
    (router, state)
}

fn gateway_client(base: String, redis: RedisPool) -> Arc<GatewayClient> {
    GatewayClient::new(
        GatewayConfig {
            base_url: base,
            master_key: None,
        },
        redis,
    )
}

async fn redis_ready() -> Option<RedisPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let pool = cache::init_pool(&url).ok()?;
    cache::verify(&pool).await.ok()?;
    Some(pool)
}

fn nonce() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}

async fn hget_u64(pool: &RedisPool, key: &str, field: &str) -> u64 {
    let mut conn = pool.get().await.unwrap();
    let value: Option<String> = deadpool_redis::redis::cmd("HGET")
        .arg(key)
        .arg(field)
        .query_async(&mut conn)
        .await
        .unwrap();
    value.and_then(|s| s.parse().ok()).unwrap_or(0)
}

fn completion_req(model: &str, content: &str) -> CompletionRequest {
    CompletionRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: content.to_string(),
        }],
        params: serde_json::Map::new(),
    }
}

#[tokio::test]
async fn completion_cache_miss_calls_proxy() {
    let Some(redis) = redis_ready().await else {
        eprintln!("SKIP: no redis; completion_cache_miss_calls_proxy skipped");
        return;
    };
    let (router, exec) = mock_exec_proxy();
    let proxy = spawn_router(router).await;
    let gw = gateway_client(format!("http://{proxy}"), redis);

    let result = gw
        .complete(completion_req("gpt-4o", &format!("ping {}", nonce())))
        .await
        .unwrap();

    assert!(!result.cached);
    assert_eq!(result.content, "hello from mock");
    assert_eq!(exec.completion_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn completion_exact_match_served_from_cache() {
    let Some(redis) = redis_ready().await else {
        eprintln!("SKIP: no redis; completion_exact_match_served_from_cache skipped");
        return;
    };
    let (router, exec) = mock_exec_proxy();
    let proxy = spawn_router(router).await;
    let gw = gateway_client(format!("http://{proxy}"), redis);
    let req = completion_req("gpt-4o", &format!("cache me {}", nonce()));

    let first = gw.complete(req.clone()).await.unwrap();
    assert!(!first.cached);
    let second = gw.complete(req.clone()).await.unwrap();
    assert!(second.cached);
    assert_eq!(second.content, first.content);
    // The proxy was hit exactly once across both calls.
    assert_eq!(exec.completion_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn completion_counters_increment_per_billed_request() {
    let Some(redis) = redis_ready().await else {
        eprintln!("SKIP: no redis; completion_counters_increment_per_billed_request skipped");
        return;
    };
    let (router, _exec) = mock_exec_proxy();
    let proxy = spawn_router(router).await;
    let gw = gateway_client(format!("http://{proxy}"), redis.clone());

    // Unique model id → its counter hash starts empty.
    let model = format!("test-model-{}", nonce());
    let req = completion_req(&model, "count me");

    let _ = gw.complete(req.clone()).await.unwrap(); // billed miss
    let _ = gw.complete(req.clone()).await.unwrap(); // cache hit (not billed)

    let model_key = format!("usage:model:{model}");
    assert_eq!(hget_u64(&redis, &model_key, "prompt_tokens").await, 5);
    assert_eq!(hget_u64(&redis, &model_key, "completion_tokens").await, 3);
    assert_eq!(hget_u64(&redis, &model_key, "requests").await, 1);
}

#[tokio::test]
async fn invalid_model_for_provider_returns_gw002() {
    let Some(redis) = redis_ready().await else {
        eprintln!("SKIP: no redis; invalid_model_for_provider_returns_gw002 skipped");
        return;
    };
    let (router, _exec) = mock_exec_proxy();
    let proxy = spawn_router(router).await;
    let gw = gateway_client(format!("http://{proxy}"), redis);

    let err = gw
        .complete(completion_req("invalid-model", &format!("x {}", nonce())))
        .await
        .unwrap_err();
    assert_eq!(err.code(), "GW002");
}

#[tokio::test]
async fn embedding_returns_vector() {
    let Some(redis) = redis_ready().await else {
        eprintln!("SKIP: no redis; embedding_returns_vector skipped");
        return;
    };
    let (router, exec) = mock_exec_proxy();
    let proxy = spawn_router(router).await;
    let gw = gateway_client(format!("http://{proxy}"), redis);

    let result = gw
        .embed(EmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
            input: format!("embed me {}", nonce()),
        })
        .await
        .unwrap();

    assert_eq!(result.embedding.len(), 3);
    assert_eq!(exec.embedding_calls.load(Ordering::SeqCst), 1);
}
