/**
 * Run dispatch + the F09 execution-event wire contract (F10).
 *
 * `startRun` kicks off a run for the live canvas graph; the returned `runId`
 * is then used to open the SSE events stream (see {@link ../hooks/useRunStream}).
 * The event types mirror the backend `runs::model::ExecutionEvent` serialisation:
 * every event's JSON carries a top-level `seq` (flattened from `SeqEvent`), the
 * `event` name, `runId`, and the variant's payload fields.
 */

import { apiGet, apiPost } from "./apiClient";
import type { FlowGraph } from "./flowGraph";

/** The accepted-run envelope returned by `POST /runs`. */
export interface RunAccepted {
  runId: string;
  status: string;
  nodeCount: number;
}

/** Request body for starting a run against the live canvas. */
export interface StartRunInput {
  prompt: string;
  graph: FlowGraph;
  /** The open flow's id when one is loaded; keys F09's in-progress guard. */
  flowId?: string;
}

/** The ordered SSE event names emitted by F09 (used to register listeners). */
export const RUN_EVENT_NAMES = [
  "run.started",
  "node.started",
  "node.partial",
  "node.completed",
  "node.failed",
  "node.skipped",
  "run.finished",
] as const;

export type RunEventName = (typeof RUN_EVENT_NAMES)[number];

export interface RunStartedEvent {
  event: "run.started";
  seq: number;
  runId: string;
  flowId?: string;
  nodeCount: number;
  rootNodeId: string;
  startedAt: string;
}

export interface NodeStartedEvent {
  event: "node.started";
  seq: number;
  runId: string;
  nodeId: string;
  agentId: string;
  agentName: string;
  model: string;
  startedAt: string;
}

export interface NodePartialEvent {
  event: "node.partial";
  seq: number;
  runId: string;
  nodeId: string;
  delta: string;
}

export interface NodeCompletedEvent {
  event: "node.completed";
  seq: number;
  runId: string;
  nodeId: string;
  output: string;
  /** Retrieval/usage metadata (snake_case on the wire); not rendered by F10. */
  retrieval?: Record<string, unknown>;
  usage?: Record<string, unknown>;
  completedAt: string;
}

export interface NodeFailedEvent {
  event: "node.failed";
  seq: number;
  runId: string;
  nodeId: string;
  error: { code: string; message: string };
  failedAt: string;
}

export interface NodeSkippedEvent {
  event: "node.skipped";
  seq: number;
  runId: string;
  nodeId: string;
  reason: string;
  upstreamNodeId: string;
}

export interface RunFinishedEvent {
  event: "run.finished";
  seq: number;
  runId: string;
  status: "succeeded" | "failed";
  output: string | null;
  failedNodes: string[];
  skippedNodes: string[];
  finishedAt: string;
}

/** The discriminated union of all execution events, keyed by `event`. */
export type RunEvent =
  | RunStartedEvent
  | NodeStartedEvent
  | NodePartialEvent
  | NodeCompletedEvent
  | NodeFailedEvent
  | NodeSkippedEvent
  | RunFinishedEvent;

/** Snapshot returned by `GET /runs/{id}` — the buffered log + terminal output. */
export interface RunSnapshot {
  runId: string;
  status: "running" | "succeeded" | "failed";
  output: string | null;
  events: RunEvent[];
}

/**
 * Start a run for the supplied graph + prompt (`POST /runs`). Resolves to the
 * accepted-run envelope, or throws an `ApiClientError` carrying the rejection
 * code (`RUN001`/`RUN002`/`RUN003`) the caller maps to a system line.
 */
export function startRun(input: StartRunInput): Promise<RunAccepted> {
  return apiPost<RunAccepted>("/runs", input);
}

/** Fetch a run's terminal snapshot for reconnect recovery (`GET /runs/{id}`). */
export function fetchRunSnapshot(runId: string): Promise<RunSnapshot> {
  return apiGet<RunSnapshot>(`/runs/${runId}`);
}
