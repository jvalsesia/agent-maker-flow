//! SSE streaming base: a heartbeat endpoint establishing the event contract
//! reused by execution-streaming features (F09/F10).

use std::convert::Infallible;
use std::time::Duration;

use axum::response::sse::{Event, KeepAlive, Sse};
use futures::{Stream, StreamExt};
use serde_json::json;
use tokio_stream::wrappers::IntervalStream;

/// `GET /api/v1/sse/heartbeat`
///
/// Emits a named `ping` event with an incrementing `seq` payload. The first
/// tick fires immediately, then every 15 seconds.
pub async fn heartbeat() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(15)))
        .enumerate()
        .map(|(i, _)| {
            let seq = (i as u64) + 1;
            Ok(Event::default()
                .event("ping")
                .data(json!({ "seq": seq }).to_string()))
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
