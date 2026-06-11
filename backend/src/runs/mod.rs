//! Flow execution engine (F09).
//!
//! Takes a flow graph plus a user prompt, translates the graph into a DAG,
//! executes its agent nodes in dependency order, forwards each node's output
//! to its downstream nodes, and streams ordered execution events over SSE.
//!
//! - `model` — run identity, statuses, the execution-event contract.
//! - `graph` — pure `FlowGraph` → `Dag` translation + acyclic/single-root
//!   validation.
//! - `registry` — in-memory run store: event log buffering, broadcast fan-out,
//!   status, per-`flowId` in-progress guard.
//! - `engine` — topological concurrent scheduler, per-node execution, event
//!   emission, partial-failure skip propagation.
//! - `service` — request validation, agent pre-resolution, run spawning, SSE
//!   subscription assembly.

pub mod engine;
pub mod graph;
pub mod model;
pub mod registry;
pub mod service;

pub use model::{
    ExecutionEvent, NodeOutcome, RunRequest, RunStatus, SeqEvent, RUN_STARTED, RUN_FINISHED,
    NODE_STARTED, NODE_PARTIAL, NODE_COMPLETED, NODE_FAILED, NODE_SKIPPED,
};
pub use registry::RunRegistry;
