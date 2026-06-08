//! F02 auth integration tests.
//!
//! A locally generated RSA key is injected into the JWKS cache and the config's
//! issuer / authorized parties are pinned to test values, so a locally signed
//! JWT verifies without contacting Clerk. Tests that exercise the `/me` success
//! path or a streaming SSE connection drive the JIT user upsert and therefore
//! soft-skip when no database is reachable (consistent with `health_test`).

use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use agent_maker_flow_backend::{
    app,
    auth::AuthState,
    cache,
    config::{AppConfig, ClerkConfig},
    db,
    state::AppState,
};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
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

/// Sign a JWT with the test private key and the test `kid`.
fn make_token(claims: &Value) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_string());
    let key = EncodingKey::from_rsa_pem(PRIV_PEM).expect("encoding key");
    encode(&header, claims, &key).expect("encode token")
}

fn valid_claims(sub: &str) -> Value {
    json!({
        "sub": sub,
        "iss": ISSUER,
        "azp": AZP,
        "exp": unix_now() + 3600,
        "email": "joao@example.com",
    })
}

fn build_state(db: PgPool) -> AppState {
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

    AppState {
        db,
        redis,
        config: AppConfig {
            database_url: String::new(),
            redis_url,
            bind_addr: "127.0.0.1:0".to_string(),
            frontend_origin: AZP.to_string(),
            clerk,
        },
        auth,
    }
}

/// A lazy pool that never connects unless a handler actually queries it — lets
/// the 401 path (which rejects before any DB access) run without a database.
fn lazy_db() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://amf:amf@localhost:5432/amf".into());
    PgPoolOptions::new().connect_lazy(&url).expect("lazy pool")
}

/// A reachable, migrated pool, or `None` when no database is available.
async fn try_db() -> Option<PgPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = db::init_pool(&url).await.ok()?;
    db::run_migrations(&pool).await.ok()?;
    Some(pool)
}

async fn spawn(state: AppState) -> SocketAddr {
    let router = app::build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    addr
}

#[tokio::test]
async fn me_without_token_returns_401() {
    let addr = spawn(build_state(lazy_db())).await;
    let resp = reqwest::get(format!("http://{addr}/api/v1/me")).await.unwrap();
    assert_eq!(resp.status(), 401);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "error");
    assert_eq!(body["error"]["code"], "AUTH001");
}

#[tokio::test]
async fn me_with_malformed_token_returns_401() {
    let addr = spawn(build_state(lazy_db())).await;
    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/me"))
        .bearer_auth("not-a-jwt")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "AUTH001");
}

#[tokio::test]
async fn me_with_expired_token_returns_401() {
    let addr = spawn(build_state(lazy_db())).await;
    let claims = json!({
        "sub": "user_expired",
        "iss": ISSUER,
        "azp": AZP,
        // Well past the default 60s validation leeway.
        "exp": unix_now() - 3600,
    });
    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/me"))
        .bearer_auth(make_token(&claims))
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
async fn me_with_wrong_azp_returns_401() {
    let addr = spawn(build_state(lazy_db())).await;
    let claims = json!({
        "sub": "user_badazp",
        "iss": ISSUER,
        "azp": "https://evil.example",
        "exp": unix_now() + 3600,
    });
    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/me"))
        .bearer_auth(make_token(&claims))
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
async fn me_with_valid_token_returns_user() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database reachable; me_with_valid_token_returns_user skipped");
        return;
    };
    let addr = spawn(build_state(db)).await;
    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/me"))
        .bearer_auth(make_token(&valid_claims("user_valid_1")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "success");
    assert_eq!(body["data"]["user_id"], "user_valid_1");
}

#[tokio::test]
async fn health_remains_public() {
    let addr = spawn(build_state(lazy_db())).await;
    let resp = reqwest::get(format!("http://{addr}/api/v1/health"))
        .await
        .unwrap();
    // Health is outside the auth layer: it must never be rejected with 401.
    // (Returns 200 with a DB, 503 without — either way, not unauthorized.)
    assert_ne!(resp.status(), 401);
}

#[tokio::test]
async fn sse_heartbeat_without_token_refused() {
    let addr = spawn(build_state(lazy_db())).await;
    let resp = reqwest::get(format!("http://{addr}/api/v1/sse/heartbeat"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "AUTH001");
    // The error envelope is JSON, never an SSE event frame.
    assert!(!body.to_string().contains("event: ping"));
}

#[tokio::test]
async fn sse_heartbeat_with_valid_token_streams() {
    let Some(db) = try_db().await else {
        eprintln!("SKIP: no database reachable; sse_heartbeat_with_valid_token_streams skipped");
        return;
    };
    let addr = spawn(build_state(db)).await;
    let token = make_token(&valid_claims("user_sse_1"));
    let mut resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/sse/heartbeat?token={token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let chunk = tokio::time::timeout(Duration::from_secs(5), resp.chunk())
        .await
        .expect("timed out waiting for first SSE chunk")
        .expect("stream error")
        .expect("stream ended without data");
    let text = String::from_utf8_lossy(&chunk);
    assert!(text.contains("event: ping"), "first chunk was: {text}");
}
