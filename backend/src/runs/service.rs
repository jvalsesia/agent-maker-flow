//! Run service: validate the request, pre-resolve every node's agent, apply
//! the in-progress guard, register the run, and spawn the engine.
//!
//! The HTTP layer ([`crate::routes::runs`]) calls into this module and never
//! touches the registry or engine directly. SSE stream assembly lives here
//! too: [`into_sse_stream`] turns a [`Subscription`] into the
//! buffered-replay-then-live event stream the route handler hands to
//! `axum::response::sse::Sse`.

use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::sync::Arc;

use axum::response::sse::Event;
use futures::stream::{self, Stream};
use sqlx::PgPool;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::agents;
use crate::error::AppError;
use crate::gateway::GatewayClient;
use crate::runs::engine::{self, EngineInputs};
use crate::runs::graph::{self, Dag};
use crate::runs::model::{RunRequest, SeqEvent, PROMPT_MAX};
use crate::runs::registry::{RunRegistry, Subscription};

/// Outcome of a successful start request.
#[derive(Debug, Clone)]
pub struct StartedRun {
    pub run_id: Uuid,
    pub node_count: usize,
}

fn validation(message: impl Into<String>) -> AppError {
    AppError::Validation {
        code: "RUN001",
        message: message.into(),
    }
}

/// Trim + length-validate the prompt.
fn validate_prompt(prompt: &str) -> Result<String, AppError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return Err(validation("Prompt is required."));
    }
    if trimmed.chars().count() > PROMPT_MAX {
        return Err(validation(format!(
            "Prompt must be {PROMPT_MAX} characters or fewer."
        )));
    }
    Ok(trimmed.to_string())
}

/// Pre-resolve every node's agent. A node referencing a missing or
/// other-user agent rejects the run with `RunAgentMissing` (`RUN003`),
/// mirroring the F07/F08 "agent missing" semantics so the user fixes the
/// graph before any node executes.
async fn resolve_agents(
    db: &PgPool,
    owner_id: &str,
    dag: &Dag,
) -> Result<HashMap<String, agents::Agent>, AppError> {
    let mut out: HashMap<String, agents::Agent> = HashMap::with_capacity(dag.nodes.len());
    for node_id in &dag.nodes {
        let agent_id = *dag.agent_of.get(node_id).expect("agent_of populated");
        let agent = agents::repo::get(db, owner_id, agent_id)
            .await?
            .ok_or(AppError::RunAgentMissing)?;
        out.insert(node_id.clone(), agent);
    }
    Ok(out)
}

/// Validate, translate, pre-resolve agents, register, and spawn the engine.
/// Returns the new run id and node count so the route handler can render the
/// `data.runId`/`data.nodeCount` body. Errors map to `RUN001` (validation),
/// `RUN002` (in-progress guard), `RUN003` (missing agent).
pub async fn start(
    db: PgPool,
    gateway: Arc<GatewayClient>,
    registry: Arc<RunRegistry>,
    owner_id: &str,
    request: RunRequest,
) -> Result<StartedRun, AppError> {
    let prompt = validate_prompt(&request.prompt)?;
    let dag = graph::translate(&request.graph)?;
    let agents_map = resolve_agents(&db, owner_id, &dag).await?;

    let node_count = dag.nodes.len();
    let (run_id, _handle) = registry.register(owner_id, request.flow_id)?;

    let inputs = EngineInputs {
        run_id,
        owner_id: owner_id.to_string(),
        flow_id: request.flow_id,
        prompt,
        history: Arc::new(request.history),
        dag,
        agents: Arc::new(agents_map),
    };

    let registry_clone = registry.clone();
    let gateway_clone = gateway.clone();
    let db_clone = db.clone();
    tokio::spawn(async move {
        engine::execute(inputs, db_clone, gateway_clone, registry_clone).await;
    });

    Ok(StartedRun { run_id, node_count })
}

/// Serialise one [`SeqEvent`] into an SSE `Event` with the right name + id.
fn seq_event_to_sse(seq_event: &SeqEvent) -> Result<Event, Infallible> {
    let data = serde_json::to_string(seq_event).unwrap_or_else(|_| "{}".to_string());
    Ok(Event::default()
        .id(seq_event.seq.to_string())
        .event(seq_event.event.name())
        .data(data))
}

/// State for the unfold stream that drives the SSE response: drain the
/// buffered replay first, then read from the live broadcast receiver until
/// the terminal `run.finished` event lands (or the channel closes).
enum SseState {
    Replay {
        queue: VecDeque<SeqEvent>,
        live: Option<broadcast::Receiver<SeqEvent>>,
    },
    Live(broadcast::Receiver<SeqEvent>),
    Done,
}

/// Convert a [`Subscription`] into the SSE event stream. Replays the
/// buffered log (in order) then streams live from the broadcast channel
/// until the terminal `run.finished` event lands.
pub fn into_sse_stream(
    subscription: Subscription,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let Subscription {
        replay,
        receiver,
        terminal,
    } = subscription;
    let initial = SseState::Replay {
        queue: VecDeque::from(replay),
        live: if terminal { None } else { Some(receiver) },
    };
    stream::unfold(initial, |state| async move {
        match state {
            SseState::Replay { mut queue, live } => {
                if let Some(ev) = queue.pop_front() {
                    let item = seq_event_to_sse(&ev);
                    let next = if ev.event.is_terminal() {
                        SseState::Done
                    } else {
                        SseState::Replay { queue, live }
                    };
                    return Some((item, next));
                }
                match live {
                    Some(rx) => recv_next(rx).await,
                    None => None,
                }
            }
            SseState::Live(rx) => recv_next(rx).await,
            SseState::Done => None,
        }
    })
}

/// Wait for the next live event; stop after the terminal one or when the
/// channel closes. Lagged events are skipped so a slow client doesn't see
/// a full failure — the replay path covers them on reconnect.
async fn recv_next(
    mut rx: broadcast::Receiver<SeqEvent>,
) -> Option<(Result<Event, Infallible>, SseState)> {
    loop {
        match rx.recv().await {
            Ok(ev) => {
                let item = seq_event_to_sse(&ev);
                let next = if ev.event.is_terminal() {
                    SseState::Done
                } else {
                    SseState::Live(rx)
                };
                return Some((item, next));
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runs::model::ExecutionEvent;
    use chrono::Utc;
    use futures::StreamExt;
    use uuid::Uuid;

    #[test]
    fn validate_prompt_trims() {
        assert_eq!(validate_prompt("  hello  ").unwrap(), "hello");
    }

    #[test]
    fn validate_prompt_rejects_empty() {
        let err = validate_prompt("   ").unwrap_err();
        assert_eq!(err.code(), "RUN001");
    }

    #[test]
    fn validate_prompt_rejects_oversized() {
        let big = "a".repeat(PROMPT_MAX + 1);
        let err = validate_prompt(&big).unwrap_err();
        assert_eq!(err.code(), "RUN001");
    }

    fn started(run_id: Uuid, root: &str) -> SeqEvent {
        SeqEvent {
            seq: 1,
            event: ExecutionEvent::RunStarted {
                run_id,
                flow_id: None,
                node_count: 1,
                root_node_id: root.to_string(),
                started_at: Utc::now(),
            },
        }
    }

    fn finished(run_id: Uuid) -> SeqEvent {
        SeqEvent {
            seq: 2,
            event: ExecutionEvent::RunFinished {
                run_id,
                status: crate::runs::model::RunStatus::Succeeded,
                output: Some("done".into()),
                failed_nodes: vec![],
                skipped_nodes: vec![],
                finished_at: Utc::now(),
            },
        }
    }

    #[tokio::test]
    async fn into_sse_stream_emits_replay_then_stops_at_terminal() {
        let run_id = Uuid::new_v4();
        let (tx, rx) = tokio::sync::broadcast::channel::<SeqEvent>(8);
        // Replay carries both events including the terminal one.
        let sub = Subscription {
            replay: vec![started(run_id, "n1"), finished(run_id)],
            receiver: rx,
            terminal: true,
        };
        // Drop the sender to ensure no live events are needed.
        drop(tx);
        let stream = into_sse_stream(sub);
        let collected: Vec<_> = stream.collect().await;
        // Two SSE events emitted, then the stream ends.
        assert_eq!(collected.len(), 2);
    }

    #[tokio::test]
    async fn into_sse_stream_falls_through_to_live() {
        let run_id = Uuid::new_v4();
        let (tx, rx) = tokio::sync::broadcast::channel::<SeqEvent>(8);
        let sub = Subscription {
            replay: vec![started(run_id, "n1")],
            receiver: rx,
            terminal: false,
        };
        let mut stream = Box::pin(into_sse_stream(sub));
        // First pull: replay event.
        let _first = stream.next().await.expect("replay event");
        // Now send a live terminal event.
        tx.send(finished(run_id)).unwrap();
        let _second = stream.next().await.expect("terminal event");
        // Stream should be done after the terminal event.
        assert!(stream.next().await.is_none());
    }
}
