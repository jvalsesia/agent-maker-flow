//! In-memory run registry.
//!
//! Holds the ordered event log, a live broadcast channel, status, and
//! aggregated output for every active or recently-finished run. Run state is
//! ephemeral (matches the PRD's "no jobs outliving the session beyond
//! terminal recovery" stance) — `Arc<RunRegistry>` lives in `AppState` and
//! every handler reaches it through there.
//!
//! Concurrency:
//! - `RunHandle` is wrapped in `Mutex` so an `append` (which both pushes to
//!   the event log and fans out on the broadcast channel) is atomic.
//! - The top-level map is wrapped in `Mutex` so `register`/`finish`/
//!   `subscribe`/`snapshot`/`evict` don't race.
//!
//! Ownership:
//! - `GET /runs/{id}` and `/events` both go through helpers that compare
//!   `owner_id` to the caller and return `AppError::NotFound` on mismatch —
//!   absent runs and other-user runs are indistinguishable.
//!
//! Eviction:
//! - Finished runs are retained up to `MAX_FINISHED_RUNS`. When the cap is
//!   exceeded the oldest finished run (by `finished_at`) is dropped so the
//!   map cannot grow unbounded.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::error::AppError;
use crate::runs::model::{ExecutionEvent, RunStatus, SeqEvent};

/// Cap on the number of finished runs retained in memory before the oldest is
/// evicted. Active (`RunStatus::Running`) runs are never evicted.
pub const MAX_FINISHED_RUNS: usize = 256;

/// Capacity for the per-run broadcast channel. The replay-on-subscribe path
/// covers events emitted before the receiver joined, so this only needs to
/// absorb live bursts; finished events stay in the buffered log.
const BROADCAST_CAPACITY: usize = 256;

/// One run's slot in the registry.
pub struct RunHandle {
    pub run_id: Uuid,
    pub owner_id: String,
    pub flow_id: Option<Uuid>,
    pub status: RunStatus,
    /// Ordered buffered log; each entry's `seq` is the SSE event id.
    pub events: Vec<SeqEvent>,
    /// Live fan-out to subscribed SSE streams.
    pub tx: broadcast::Sender<SeqEvent>,
    /// Aggregated terminal output once `run.finished` is emitted.
    pub output: Option<String>,
    /// When the terminal event landed; populated alongside `status` going
    /// non-`Running`. Used by eviction to drop the oldest finished run.
    pub finished_at: Option<DateTime<Utc>>,
    /// Per-run monotonic counter. Incremented every `append`.
    next_seq: u64,
}

impl RunHandle {
    fn new(run_id: Uuid, owner_id: String, flow_id: Option<Uuid>) -> Self {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            run_id,
            owner_id,
            flow_id,
            status: RunStatus::Running,
            events: Vec::new(),
            tx,
            output: None,
            finished_at: None,
            next_seq: 0,
        }
    }

    /// Append an event to the log + broadcast it. The `seq` becomes the SSE
    /// `id:` used by `Last-Event-ID` to resume.
    fn append(&mut self, event: ExecutionEvent) -> SeqEvent {
        self.next_seq += 1;
        let seq_event = SeqEvent {
            seq: self.next_seq,
            event,
        };
        self.events.push(seq_event.clone());
        // `send` errors only when no receiver is listening, which is fine —
        // the buffered log is the source of truth and replay catches the
        // event when a subscriber attaches later.
        let _ = self.tx.send(seq_event.clone());
        seq_event
    }
}

/// What a subscriber receives to drive an SSE stream.
pub struct Subscription {
    /// Already-buffered events strictly after `last_event_id` (or the full
    /// log when none was supplied) — replayed before live streaming begins.
    pub replay: Vec<SeqEvent>,
    /// Live fan-out from the run's broadcast channel.
    pub receiver: broadcast::Receiver<SeqEvent>,
    /// `true` when the run is already terminal — the subscriber only needs
    /// the replay; the live receiver will yield nothing further.
    pub terminal: bool,
}

/// Snapshot for `GET /runs/{id}` (terminal/status fetch on reconnect).
#[derive(Debug, Clone)]
pub struct RunSnapshot {
    pub run_id: Uuid,
    pub status: RunStatus,
    pub output: Option<String>,
    pub events: Vec<SeqEvent>,
}

/// The in-memory map of all live and recently-finished runs.
pub struct RunRegistry {
    runs: Mutex<HashMap<Uuid, Arc<Mutex<RunHandle>>>>,
}

impl Default for RunRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RunRegistry {
    pub fn new() -> Self {
        Self {
            runs: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new run if no other run owned by `owner_id` is already
    /// running for the same `flow_id`. Returns the new handle's id.
    pub fn register(
        &self,
        owner_id: &str,
        flow_id: Option<Uuid>,
    ) -> Result<(Uuid, Arc<Mutex<RunHandle>>), AppError> {
        let mut runs = self.runs.lock().expect("runs lock poisoned");
        if let Some(fid) = flow_id {
            for handle in runs.values() {
                let guard = handle.lock().expect("run handle lock poisoned");
                if guard.owner_id == owner_id
                    && guard.flow_id == Some(fid)
                    && guard.status == RunStatus::Running
                {
                    return Err(AppError::RunInProgress);
                }
            }
        }
        let run_id = Uuid::new_v4();
        let handle = Arc::new(Mutex::new(RunHandle::new(
            run_id,
            owner_id.to_string(),
            flow_id,
        )));
        runs.insert(run_id, handle.clone());
        Ok((run_id, handle))
    }

    /// Append an event to a run's log and broadcast it live.
    pub fn append(&self, run_id: Uuid, event: ExecutionEvent) -> Option<SeqEvent> {
        let handle = self.get_handle(run_id)?;
        let mut guard = handle.lock().expect("run handle lock poisoned");
        Some(guard.append(event))
    }

    /// Mark a run terminal with its aggregated output. Triggers a one-shot
    /// eviction pass so the map cannot grow unbounded.
    pub fn finish(&self, run_id: Uuid, status: RunStatus, output: Option<String>) {
        if let Some(handle) = self.get_handle(run_id) {
            {
                let mut guard = handle.lock().expect("run handle lock poisoned");
                guard.status = status;
                guard.output = output;
                guard.finished_at = Some(Utc::now());
            }
        }
        self.evict_excess_finished();
    }

    /// Subscribe an SSE client to a run, with owner enforcement and
    /// `Last-Event-ID` replay semantics. Returns 404 when the run is unknown
    /// or owned by another user (never reveals existence).
    pub fn subscribe(
        &self,
        run_id: Uuid,
        caller_id: &str,
        last_event_id: Option<u64>,
    ) -> Result<Subscription, AppError> {
        let handle = self.get_handle(run_id).ok_or(AppError::NotFound)?;
        let guard = handle.lock().expect("run handle lock poisoned");
        if guard.owner_id != caller_id {
            return Err(AppError::NotFound);
        }
        let from = last_event_id.unwrap_or(0);
        let replay: Vec<SeqEvent> = guard
            .events
            .iter()
            .filter(|e| e.seq > from)
            .cloned()
            .collect();
        let receiver = guard.tx.subscribe();
        let terminal = guard.status.is_terminal();
        Ok(Subscription {
            replay,
            receiver,
            terminal,
        })
    }

    /// Owner-scoped snapshot for `GET /runs/{id}`.
    pub fn snapshot(&self, run_id: Uuid, caller_id: &str) -> Result<RunSnapshot, AppError> {
        let handle = self.get_handle(run_id).ok_or(AppError::NotFound)?;
        let guard = handle.lock().expect("run handle lock poisoned");
        if guard.owner_id != caller_id {
            return Err(AppError::NotFound);
        }
        Ok(RunSnapshot {
            run_id: guard.run_id,
            status: guard.status,
            output: guard.output.clone(),
            events: guard.events.clone(),
        })
    }

    /// Pull a handle out of the map. Internal helper.
    fn get_handle(&self, run_id: Uuid) -> Option<Arc<Mutex<RunHandle>>> {
        self.runs
            .lock()
            .expect("runs lock poisoned")
            .get(&run_id)
            .cloned()
    }

    /// Drop the oldest finished runs above `MAX_FINISHED_RUNS`. Running runs
    /// are never evicted.
    fn evict_excess_finished(&self) {
        let mut runs = self.runs.lock().expect("runs lock poisoned");
        let mut finished: Vec<(Uuid, DateTime<Utc>)> = runs
            .iter()
            .filter_map(|(id, handle)| {
                let guard = handle.lock().expect("run handle lock poisoned");
                if guard.status.is_terminal() {
                    Some((*id, guard.finished_at.unwrap_or_else(Utc::now)))
                } else {
                    None
                }
            })
            .collect();
        if finished.len() <= MAX_FINISHED_RUNS {
            return;
        }
        finished.sort_by_key(|(_, ts)| *ts);
        let drop_count = finished.len() - MAX_FINISHED_RUNS;
        for (id, _) in finished.into_iter().take(drop_count) {
            runs.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runs::model::ExecutionEvent;
    use chrono::Utc;

    fn run_started(run_id: Uuid, root: &str) -> ExecutionEvent {
        ExecutionEvent::RunStarted {
            run_id,
            flow_id: None,
            node_count: 1,
            root_node_id: root.to_string(),
            started_at: Utc::now(),
        }
    }

    fn run_finished(run_id: Uuid) -> ExecutionEvent {
        ExecutionEvent::RunFinished {
            run_id,
            status: RunStatus::Succeeded,
            output: Some("done".to_string()),
            failed_nodes: vec![],
            skipped_nodes: vec![],
            finished_at: Utc::now(),
        }
    }

    #[test]
    fn register_returns_unique_run_id() {
        let reg = RunRegistry::new();
        let (a, _) = reg.register("user_a", None).unwrap();
        let (b, _) = reg.register("user_a", None).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn in_progress_guard_rejects_same_flow() {
        let reg = RunRegistry::new();
        let flow = Uuid::new_v4();
        let (_, _) = reg.register("user_a", Some(flow)).unwrap();
        match reg.register("user_a", Some(flow)) {
            Err(e) => assert_eq!(e.code(), "RUN002"),
            Ok(_) => panic!("expected RunInProgress"),
        }
    }

    #[test]
    fn guard_allows_when_no_flow_id() {
        // Unsaved canvases aren't keyed; concurrent runs are allowed.
        let reg = RunRegistry::new();
        reg.register("user_a", None).unwrap();
        reg.register("user_a", None).unwrap();
    }

    #[test]
    fn guard_allows_after_run_finished() {
        let reg = RunRegistry::new();
        let flow = Uuid::new_v4();
        let (first, _) = reg.register("user_a", Some(flow)).unwrap();
        reg.finish(first, RunStatus::Succeeded, None);
        reg.register("user_a", Some(flow)).unwrap();
    }

    #[test]
    fn guard_scoped_per_user() {
        let reg = RunRegistry::new();
        let flow = Uuid::new_v4();
        reg.register("user_a", Some(flow)).unwrap();
        // A different user with the same flow id is unrelated.
        reg.register("user_b", Some(flow)).unwrap();
    }

    #[test]
    fn append_assigns_monotonic_seq() {
        let reg = RunRegistry::new();
        let (run_id, _) = reg.register("user_a", None).unwrap();
        let first = reg.append(run_id, run_started(run_id, "n1")).unwrap();
        let second = reg.append(run_id, run_finished(run_id)).unwrap();
        assert_eq!(first.seq, 1);
        assert_eq!(second.seq, 2);
    }

    #[test]
    fn subscribe_replays_full_log_when_no_last_event_id() {
        let reg = RunRegistry::new();
        let (run_id, _) = reg.register("user_a", None).unwrap();
        reg.append(run_id, run_started(run_id, "n1"));
        reg.append(run_id, run_finished(run_id));
        let sub = reg.subscribe(run_id, "user_a", None).unwrap();
        assert_eq!(sub.replay.len(), 2);
        assert_eq!(sub.replay[0].seq, 1);
        assert_eq!(sub.replay[1].seq, 2);
    }

    #[test]
    fn replay_honors_last_event_id() {
        let reg = RunRegistry::new();
        let (run_id, _) = reg.register("user_a", None).unwrap();
        reg.append(run_id, run_started(run_id, "n1"));
        reg.append(run_id, run_finished(run_id));
        let sub = reg.subscribe(run_id, "user_a", Some(1)).unwrap();
        assert_eq!(sub.replay.len(), 1);
        assert_eq!(sub.replay[0].seq, 2);
    }

    #[test]
    fn snapshot_scoped_to_owner() {
        let reg = RunRegistry::new();
        let (run_id, _) = reg.register("user_a", None).unwrap();
        reg.append(run_id, run_started(run_id, "n1"));
        // Other user gets NotFound (never reveals existence).
        let err = reg.snapshot(run_id, "user_b").unwrap_err();
        assert_eq!(err.code(), "NOT_FOUND");
        // Owner sees their snapshot.
        let snap = reg.snapshot(run_id, "user_a").unwrap();
        assert_eq!(snap.events.len(), 1);
    }

    fn assert_not_found(result: Result<Subscription, AppError>) {
        match result {
            Err(e) => assert_eq!(e.code(), "NOT_FOUND"),
            Ok(_) => panic!("expected NotFound"),
        }
    }

    #[test]
    fn subscribe_scoped_to_owner() {
        let reg = RunRegistry::new();
        let (run_id, _) = reg.register("user_a", None).unwrap();
        assert_not_found(reg.subscribe(run_id, "user_b", None));
    }

    #[test]
    fn subscribe_unknown_run_is_not_found() {
        let reg = RunRegistry::new();
        assert_not_found(reg.subscribe(Uuid::new_v4(), "user_a", None));
    }

    #[test]
    fn finish_marks_status_and_output() {
        let reg = RunRegistry::new();
        let (run_id, _) = reg.register("user_a", None).unwrap();
        reg.finish(run_id, RunStatus::Succeeded, Some("hello".into()));
        let snap = reg.snapshot(run_id, "user_a").unwrap();
        assert_eq!(snap.status, RunStatus::Succeeded);
        assert_eq!(snap.output.as_deref(), Some("hello"));
    }

    #[test]
    fn subscribe_after_finish_marks_terminal() {
        let reg = RunRegistry::new();
        let (run_id, _) = reg.register("user_a", None).unwrap();
        reg.append(run_id, run_finished(run_id));
        reg.finish(run_id, RunStatus::Succeeded, Some("done".into()));
        let sub = reg.subscribe(run_id, "user_a", None).unwrap();
        assert!(sub.terminal);
        assert_eq!(sub.replay.len(), 1);
    }
}
