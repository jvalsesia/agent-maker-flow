//! Run identity, status, request body, and the tagged execution-event
//! contract.
//!
//! Each `ExecutionEvent` variant serialises with `runId` and a per-run
//! monotonic `seq`; the `seq` is reused as the SSE `id` for `Last-Event-ID`
//! reconnect. Retrieval and usage payload fields mirror the existing
//! serialisations from F06 (`memory::retrieval::RetrievalOutcome`) and F03
//! (`gateway::types::CompletionResult`) so the wire shape stays consistent.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::memory::retrieval::RetrievalStatus;

/// Max length of the run prompt (mirrors the F04 `system_prompt` cap).
pub const PROMPT_MAX: usize = 32_000;

/// SSE event names. Kept as `&'static str` so the names can be reused both as
/// the SSE `event:` tag and as the variant discriminator.
pub const RUN_STARTED: &str = "run.started";
pub const RUN_FINISHED: &str = "run.finished";
pub const NODE_STARTED: &str = "node.started";
pub const NODE_PARTIAL: &str = "node.partial";
pub const NODE_COMPLETED: &str = "node.completed";
pub const NODE_FAILED: &str = "node.failed";
pub const NODE_SKIPPED: &str = "node.skipped";

/// The lifecycle of a single run, observed via `GET /runs/{id}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Succeeded,
    Failed,
}

impl RunStatus {
    /// `true` once the run reached a terminal state.
    pub fn is_terminal(self) -> bool {
        matches!(self, RunStatus::Succeeded | RunStatus::Failed)
    }
}

/// Per-node terminal outcome captured by the engine for the `run.finished`
/// payload and the snapshot endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeOutcome {
    Completed,
    Failed,
    Skipped,
}

/// Prior conversational turn supplied by the caller; each agent's `recent_n`
/// caps how many of these reach its prompt.
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryTurn {
    pub role: String,
    pub content: String,
}

/// Request body for `POST /runs`.
#[derive(Debug, Clone, Deserialize)]
pub struct RunRequest {
    pub prompt: String,
    pub graph: Value,
    #[serde(default, rename = "flowId")]
    pub flow_id: Option<Uuid>,
    #[serde(default)]
    pub history: Vec<HistoryTurn>,
}

/// Structured failure payload for a `node.failed` event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NodeError {
    pub code: String,
    pub message: String,
}

/// Aggregated retrieval metadata surfaced in `node.completed`. Mirrors the F06
/// fields (`retrieved_count`, `excluded_mismatched`, `status`) so the wire
/// shape is shared.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RetrievalSummary {
    pub retrieved_count: usize,
    pub excluded_mismatched: usize,
    pub status: RetrievalStatus,
}

/// Aggregated usage metadata surfaced in `node.completed`. Mirrors the F03
/// `CompletionResult` shape.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UsageSummary {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cost_usd: f64,
    pub cached: bool,
}

/// One event with its per-run monotonic sequence id. The `seq` is the SSE
/// `id:` value used by `Last-Event-ID` to resume; `event` flattens its
/// own `event:` tag and payload fields up into the JSON envelope.
#[derive(Debug, Clone, Serialize)]
pub struct SeqEvent {
    pub seq: u64,
    #[serde(flatten)]
    pub event: ExecutionEvent,
}

impl SeqEvent {
    /// SSE `event:` name (delegated to the inner variant).
    pub fn name(&self) -> &'static str {
        self.event.name()
    }
}

/// The tagged execution-event enum streamed to SSE clients. Each variant
/// carries its event name and the run id so a snapshot can reconstruct the
/// full ordered log.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ExecutionEvent {
    #[serde(rename = "run.started")]
    RunStarted {
        #[serde(rename = "runId")]
        run_id: Uuid,
        #[serde(rename = "flowId", skip_serializing_if = "Option::is_none")]
        flow_id: Option<Uuid>,
        #[serde(rename = "nodeCount")]
        node_count: usize,
        #[serde(rename = "rootNodeId")]
        root_node_id: String,
        #[serde(rename = "startedAt")]
        started_at: DateTime<Utc>,
    },
    #[serde(rename = "node.started")]
    NodeStarted {
        #[serde(rename = "runId")]
        run_id: Uuid,
        #[serde(rename = "nodeId")]
        node_id: String,
        #[serde(rename = "agentId")]
        agent_id: Uuid,
        #[serde(rename = "agentName")]
        agent_name: String,
        model: String,
        #[serde(rename = "startedAt")]
        started_at: DateTime<Utc>,
    },
    #[serde(rename = "node.partial")]
    NodePartial {
        #[serde(rename = "runId")]
        run_id: Uuid,
        #[serde(rename = "nodeId")]
        node_id: String,
        delta: String,
    },
    #[serde(rename = "node.completed")]
    NodeCompleted {
        #[serde(rename = "runId")]
        run_id: Uuid,
        #[serde(rename = "nodeId")]
        node_id: String,
        output: String,
        retrieval: RetrievalSummary,
        usage: UsageSummary,
        #[serde(rename = "completedAt")]
        completed_at: DateTime<Utc>,
    },
    #[serde(rename = "node.failed")]
    NodeFailed {
        #[serde(rename = "runId")]
        run_id: Uuid,
        #[serde(rename = "nodeId")]
        node_id: String,
        error: NodeError,
        #[serde(rename = "failedAt")]
        failed_at: DateTime<Utc>,
    },
    #[serde(rename = "node.skipped")]
    NodeSkipped {
        #[serde(rename = "runId")]
        run_id: Uuid,
        #[serde(rename = "nodeId")]
        node_id: String,
        reason: String,
        #[serde(rename = "upstreamNodeId")]
        upstream_node_id: String,
    },
    #[serde(rename = "run.finished")]
    RunFinished {
        #[serde(rename = "runId")]
        run_id: Uuid,
        status: RunStatus,
        output: Option<String>,
        #[serde(rename = "failedNodes")]
        failed_nodes: Vec<String>,
        #[serde(rename = "skippedNodes")]
        skipped_nodes: Vec<String>,
        #[serde(rename = "finishedAt")]
        finished_at: DateTime<Utc>,
    },
}

impl ExecutionEvent {
    /// SSE `event:` name for this variant.
    pub fn name(&self) -> &'static str {
        match self {
            ExecutionEvent::RunStarted { .. } => RUN_STARTED,
            ExecutionEvent::NodeStarted { .. } => NODE_STARTED,
            ExecutionEvent::NodePartial { .. } => NODE_PARTIAL,
            ExecutionEvent::NodeCompleted { .. } => NODE_COMPLETED,
            ExecutionEvent::NodeFailed { .. } => NODE_FAILED,
            ExecutionEvent::NodeSkipped { .. } => NODE_SKIPPED,
            ExecutionEvent::RunFinished { .. } => RUN_FINISHED,
        }
    }

    /// `true` when this is the terminal `run.finished` event.
    pub fn is_terminal(&self) -> bool {
        matches!(self, ExecutionEvent::RunFinished { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn execution_event_names_round_trip() {
        let id = Uuid::nil();
        let ev = ExecutionEvent::RunStarted {
            run_id: id,
            flow_id: None,
            node_count: 2,
            root_node_id: "n1".to_string(),
            started_at: Utc::now(),
        };
        assert_eq!(ev.name(), RUN_STARTED);
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v.get("event").and_then(Value::as_str), Some(RUN_STARTED));
        assert_eq!(
            v.get("runId").and_then(Value::as_str),
            Some(id.to_string().as_str())
        );
    }

    #[test]
    fn run_finished_is_terminal() {
        let ev = ExecutionEvent::RunFinished {
            run_id: Uuid::nil(),
            status: RunStatus::Succeeded,
            output: Some("ok".into()),
            failed_nodes: vec![],
            skipped_nodes: vec![],
            finished_at: Utc::now(),
        };
        assert!(ev.is_terminal());
        assert!(RunStatus::Succeeded.is_terminal());
        assert!(!RunStatus::Running.is_terminal());
    }
}
