/**
 * Pure run-stream reducer (F10): folds the ordered F09 execution events into a
 * single `RunState` that drives both the canvas status badges and the monitor
 * panel. The reducer is `seq`-idempotent — an event whose `seq` is not newer
 * than the last applied one is ignored, so F09's replay-on-reconnect does not
 * double-render.
 */

import { RUN_EVENT_NAMES, type RunEvent, type RunEventName } from "./runs";

/** Per-node lifecycle, mapped 1:1 onto the canvas badge and the monitor block. */
export type NodeStatus = "idle" | "running" | "complete" | "error" | "skipped";

/** Overall run lifecycle. */
export type RunStatus = "idle" | "running" | "succeeded" | "failed";

/** Shown as the assistant turn when a run finishes with no terminal output. */
export const EMPTY_OUTPUT_MESSAGE = "The flow completed but produced no output.";

export interface NodeRunState {
  nodeId: string;
  agentName: string | null;
  model: string | null;
  status: NodeStatus;
  /** Accumulated `node.partial` deltas (intermediate output). */
  partial: string;
  /** Final text from `node.completed`. */
  output: string | null;
  /** Message from `node.failed`. */
  error: string | null;
}

export interface RunState {
  runId: string | null;
  status: RunStatus;
  nodes: Record<string, NodeRunState>;
  /** Highest applied `seq`; the idempotency guard for replayed events. */
  lastSeq: number;
  /** Aggregated terminal output once finished. */
  output: string | null;
}

/** A fresh, empty run state (before any dispatch). */
export function initialRunState(): RunState {
  return { runId: null, status: "idle", nodes: {}, lastSeq: 0, output: null };
}

/** Existing node state, or a fresh running node if an event arrives early. */
function nodeOf(state: RunState, nodeId: string): NodeRunState {
  return (
    state.nodes[nodeId] ?? {
      nodeId,
      agentName: null,
      model: null,
      status: "running",
      partial: "",
      output: null,
      error: null,
    }
  );
}

/**
 * Parse an SSE message into a typed {@link RunEvent}. `name` is the SSE
 * `event:` tag (authoritative discriminator); `data` is the raw JSON payload.
 * Returns `null` for unknown event names or malformed JSON.
 */
export function parseRunEvent(name: string, data: string): RunEvent | null {
  if (!(RUN_EVENT_NAMES as readonly string[]).includes(name)) return null;
  let payload: unknown;
  try {
    payload = JSON.parse(data);
  } catch {
    return null;
  }
  if (!payload || typeof payload !== "object") return null;
  const obj = payload as Record<string, unknown>;
  if (typeof obj.seq !== "number") return null;
  // Trust the SSE event tag as the discriminator over any `event` in the body.
  return { ...obj, event: name as RunEventName } as RunEvent;
}

/**
 * Fold one execution event into the run state. Pure: returns a new state and
 * never mutates the input. Ignores already-applied `seq` values (replay guard).
 */
export function reduceRunEvent(state: RunState, event: RunEvent): RunState {
  if (event.seq <= state.lastSeq) return state;
  const base: RunState = { ...state, lastSeq: event.seq };

  switch (event.event) {
    case "run.started":
      return { ...base, runId: event.runId, status: "running", nodes: {}, output: null };

    case "node.started": {
      const node: NodeRunState = {
        nodeId: event.nodeId,
        agentName: event.agentName,
        model: event.model,
        status: "running",
        partial: "",
        output: null,
        error: null,
      };
      return { ...base, nodes: { ...state.nodes, [event.nodeId]: node } };
    }

    case "node.partial": {
      const prev = nodeOf(state, event.nodeId);
      return {
        ...base,
        nodes: { ...state.nodes, [event.nodeId]: { ...prev, partial: prev.partial + event.delta } },
      };
    }

    case "node.completed": {
      const prev = nodeOf(state, event.nodeId);
      return {
        ...base,
        nodes: {
          ...state.nodes,
          [event.nodeId]: { ...prev, status: "complete", output: event.output },
        },
      };
    }

    case "node.failed": {
      const prev = nodeOf(state, event.nodeId);
      return {
        ...base,
        nodes: {
          ...state.nodes,
          [event.nodeId]: {
            ...prev,
            status: "error",
            error: event.error?.message ?? "Node failed.",
          },
        },
      };
    }

    case "node.skipped": {
      const prev = nodeOf(state, event.nodeId);
      return {
        ...base,
        nodes: { ...state.nodes, [event.nodeId]: { ...prev, status: "skipped" } },
      };
    }

    case "run.finished":
      return { ...base, status: event.status, output: event.output ?? null };

    default:
      return state;
  }
}

/** Derive the per-node status map threaded into the canvas node context. */
export function nodeStatusMap(state: RunState): Record<string, NodeStatus> {
  const out: Record<string, NodeStatus> = {};
  for (const [id, node] of Object.entries(state.nodes)) out[id] = node.status;
  return out;
}
