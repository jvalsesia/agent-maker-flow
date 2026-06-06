//! SSE heartbeat contract. Mounts only the heartbeat route (no DB/cache state),
//! so it runs without external dependencies.

use std::net::SocketAddr;
use std::time::Duration;

use agent_maker_flow_backend::sse;
use axum::routing::get;
use axum::Router;

#[tokio::test]
async fn heartbeat_emits_ping_events() {
    let router: Router = Router::new().route("/api/v1/sse/heartbeat", get(sse::heartbeat));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/api/v1/sse/heartbeat"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        content_type.starts_with("text/event-stream"),
        "unexpected content-type: {content_type}"
    );

    let mut resp = resp;
    let chunk = tokio::time::timeout(Duration::from_secs(5), resp.chunk())
        .await
        .expect("timed out waiting for first SSE chunk")
        .expect("stream error")
        .expect("stream ended without data");

    let text = String::from_utf8_lossy(&chunk);
    assert!(text.contains("event: ping"), "first chunk was: {text}");
    assert!(text.contains("\"seq\""), "first chunk was: {text}");
}
