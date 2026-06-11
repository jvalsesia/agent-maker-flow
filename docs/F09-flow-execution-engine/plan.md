# Implementation Plan: Flow Execution Engine

**Prerequisites:**
- Implemented dependencies: F03 LLM Gateway (`gateway::GatewayClient::complete`), F06 Vector Retrieval (`memory::retrieval::retrieve`), F07/F08 graph types (`flows::model::FlowGraph`), F04 agents (`agents::repo::get`), F02 auth (`auth::AuthUser`, `require_auth`).
- Rust toolchain with the existing backend crate (`axum 0.8.9`, `tokio`, `sqlx`, `futures`, `tokio-stream`, `uuid`, `chrono`, `serde`); `tokio::sync::broadcast` for live fan-out.
- A running LiteLLM proxy for manual runs; tests use the in-test mock proxy pattern from `tests/gateway_test.rs` / `tests/flows_test.rs`.
- Environment unchanged — F09 adds no migration and no new env vars; run state is in-memory.

## Stage 1: Run model, event contract, and DAG translation

**1. Run module scaffold and error variants** - Create the `runs` module (`mod.rs`) following the `flows` module layout, declare it in `lib.rs`, and add the three execution error variants to `error.rs` with their stable codes and HTTP statuses. Reference the spec's Component Overview and error-code tables.

**2. Run model and execution-event contract** - Define the run identity, status, request body, and the tagged execution-event type with its per-run sequence and payload fields exactly as the spec's event table and message-assembly notes describe. Reuse F06 retrieval-outcome fields and F03 usage fields in the relevant payloads so the wire shape matches their existing serialization.

**3. DAG translation and validation** - Build the pure graph module that turns a `FlowGraph` into the executor's DAG view — adjacency, in-degrees, the single root, and terminal nodes — and validates the graph is non-empty, single-rooted, and acyclic, rejecting otherwise. Cover translation and every rejection case with unit tests per the spec's testing strategy.

## Stage 2: In-memory run registry

**4. Run registry with buffered log and broadcast** - Implement the in-memory registry that stores each run's handle (owner, optional flow id, status, ordered event log, broadcast sender, terminal output), appends events atomically to both the log and the live channel, and exposes subscribe/replay/snapshot helpers. Reference the spec's `RunHandle`/`RunRegistry` data model.

**5. Concurrency guard, ownership, and eviction** - Add the per-flow in-progress guard, owner-scoped access for snapshot/events, and bounded retention of finished runs so the map cannot grow without limit. Unit-test the guard, ownership rejection, and replay-honors-`Last-Event-ID` behavior.

## Stage 3: Execution engine

**6. Topological concurrent scheduler** - Implement the executor that drives the DAG: run all currently-ready nodes concurrently, unlock successors as predecessors complete, and stop a path by marking a failed node's transitive downstream as skipped while letting unaffected branches finish. Emit the ordered lifecycle events through the registry as the run progresses.

**7. Per-node execution and message assembly** - For each node, resolve its agent config, run F06 retrieval (top-K from the agent) on the node's input, assemble the messages (preamble + system prompt, recent-N history, retrieved context ahead of forwarded upstream output), call the F03 gateway with the agent's model, and forward the produced output downstream. Aggregate the terminal nodes' outputs into the run result. Reference the spec's message-assembly notes.

**8. Engine event emission and partial-failure reporting** - Wire the engine to emit `run.started`, `node.started`, `node.partial` (full text in this version), `node.completed` (with retrieval + usage metadata), `node.failed`, `node.skipped`, and the terminal `run.finished` carrying overall status and aggregated output. Unit-test scheduling order, forwarding, skip propagation, and aggregation with stubbed node execution.

## Stage 4: Service, routes, state wiring, and integration tests

**9. Run service orchestration** - Implement the service that validates the request, pre-resolves every node's agent (rejecting a missing agent), applies the in-progress guard, registers the run, spawns the engine as a background task, and builds the SSE response stream from a registry subscription with replay. Unit-test validation and agent pre-resolution.

**10. Routes and application state** - Add `routes/runs.rs` with the start, events (SSE), and snapshot handlers rendering the platform envelope, mount them on the protected router, add the registry to `AppState`, and construct it at boot in `app.rs`. Reference the spec's API contracts.

**11. Integration tests** - Add `tests/runs_test.rs` reusing the mock-proxy + signed-token harness to cover start, ordered event streaming, output forwarding, node-failure skip + terminal failure, independent-branch continuation, invalid-DAG rejection, in-progress 409, missing-agent rejection, reconnect replay, and ownership/auth scoping, mapping to the acceptance and cross-feature criteria in the spec.
