//! Boot fail-fast behavior: dependency connectivity must error when a service
//! is unreachable. Port 1 is reserved and refuses connections immediately.

use agent_maker_flow_backend::{cache, db};

#[tokio::test]
async fn boot_aborts_when_database_unreachable() {
    let result = db::init_pool("postgres://amf:amf@127.0.0.1:1/amf").await;
    assert!(
        result.is_err(),
        "expected PostgreSQL connect to fail on a refused port"
    );
}

#[tokio::test]
async fn boot_aborts_when_redis_unreachable() {
    let pool = cache::init_pool("redis://127.0.0.1:1").expect("pool builder should succeed");
    let result = cache::verify(&pool).await;
    assert!(
        result.is_err(),
        "expected redis PING to fail on a refused port"
    );
}
