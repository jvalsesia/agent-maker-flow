import { describe, expect, it } from "vitest";

import {
  initialRunState,
  nodeStatusMap,
  parseRunEvent,
  reduceRunEvent,
  type RunState,
} from "./runStream";
import type {
  NodeCompletedEvent,
  NodeFailedEvent,
  NodePartialEvent,
  NodeSkippedEvent,
  NodeStartedEvent,
  RunFinishedEvent,
  RunStartedEvent,
} from "./runs";

const runStarted: RunStartedEvent = {
  event: "run.started",
  seq: 1,
  runId: "run-1",
  nodeCount: 2,
  rootNodeId: "n1",
  startedAt: "2026-06-12T12:00:00Z",
};

const nodeStarted = (seq: number, nodeId: string): NodeStartedEvent => ({
  event: "node.started",
  seq,
  runId: "run-1",
  nodeId,
  agentId: "a1",
  agentName: "Researcher",
  model: "gpt-4o",
  startedAt: "2026-06-12T12:00:00Z",
});

/** Apply a list of events from a fresh state, in order. */
function fold(...events: Parameters<typeof reduceRunEvent>[1][]): RunState {
  return events.reduce(reduceRunEvent, initialRunState());
}

describe("reduceRunEvent", () => {
  it("run_started_seeds_running_state", () => {
    const state = fold(runStarted);
    expect(state.status).toBe("running");
    expect(state.runId).toBe("run-1");
  });

  it("node_started_marks_node_running", () => {
    const state = fold(runStarted, nodeStarted(2, "n1"));
    expect(state.nodes.n1.status).toBe("running");
    expect(state.nodes.n1.agentName).toBe("Researcher");
    expect(state.nodes.n1.model).toBe("gpt-4o");
  });

  it("node_partial_accumulates_delta", () => {
    const partial = (seq: number, delta: string): NodePartialEvent => ({
      event: "node.partial",
      seq,
      runId: "run-1",
      nodeId: "n1",
      delta,
    });
    const state = fold(runStarted, nodeStarted(2, "n1"), partial(3, "Hello "), partial(4, "world"));
    expect(state.nodes.n1.partial).toBe("Hello world");
  });

  it("node_completed_sets_output", () => {
    const completed: NodeCompletedEvent = {
      event: "node.completed",
      seq: 3,
      runId: "run-1",
      nodeId: "n1",
      output: "final text",
      completedAt: "2026-06-12T12:00:02Z",
    };
    const state = fold(runStarted, nodeStarted(2, "n1"), completed);
    expect(state.nodes.n1.status).toBe("complete");
    expect(state.nodes.n1.output).toBe("final text");
  });

  it("node_failed_records_error", () => {
    const failed: NodeFailedEvent = {
      event: "node.failed",
      seq: 3,
      runId: "run-1",
      nodeId: "n1",
      error: { code: "GATEWAY", message: "Model gateway unavailable." },
      failedAt: "2026-06-12T12:00:02Z",
    };
    const state = fold(runStarted, nodeStarted(2, "n1"), failed);
    expect(state.nodes.n1.status).toBe("error");
    expect(state.nodes.n1.error).toBe("Model gateway unavailable.");
  });

  it("node_skipped_marks_skipped", () => {
    const skipped: NodeSkippedEvent = {
      event: "node.skipped",
      seq: 3,
      runId: "run-1",
      nodeId: "n2",
      reason: "upstream failed",
      upstreamNodeId: "n1",
    };
    const state = fold(runStarted, nodeStarted(2, "n2"), skipped);
    expect(state.nodes.n2.status).toBe("skipped");
  });

  it("run_finished_sets_terminal_output", () => {
    const finished: RunFinishedEvent = {
      event: "run.finished",
      seq: 4,
      runId: "run-1",
      status: "succeeded",
      output: "aggregated",
      failedNodes: [],
      skippedNodes: [],
      finishedAt: "2026-06-12T12:00:05Z",
    };
    const state = fold(runStarted, nodeStarted(2, "n1"), finished);
    expect(state.status).toBe("succeeded");
    expect(state.output).toBe("aggregated");
  });

  it("run_finished_empty_output_flag", () => {
    const finished: RunFinishedEvent = {
      event: "run.finished",
      seq: 2,
      runId: "run-1",
      status: "succeeded",
      output: null,
      failedNodes: [],
      skippedNodes: [],
      finishedAt: "2026-06-12T12:00:05Z",
    };
    const state = fold(runStarted, finished);
    expect(state.status).toBe("succeeded");
    expect(state.output).toBeNull();
  });

  it("ignores_already_applied_seq", () => {
    const afterStart = fold(runStarted, nodeStarted(2, "n1"));
    // Re-apply an event with seq <= lastSeq: state must be unchanged (identity).
    const replayed = reduceRunEvent(afterStart, nodeStarted(2, "n1"));
    expect(replayed).toBe(afterStart);
    const older = reduceRunEvent(afterStart, runStarted);
    expect(older).toBe(afterStart);
  });
});

describe("parseRunEvent", () => {
  it("parse_run_event_maps_name_and_json", () => {
    const data = JSON.stringify({ seq: 1, runId: "run-1", nodeCount: 2, rootNodeId: "n1" });
    const parsed = parseRunEvent("run.started", data);
    expect(parsed?.event).toBe("run.started");
    expect(parsed?.seq).toBe(1);
  });

  it("returns null for an unknown event name", () => {
    expect(parseRunEvent("bogus", "{}")).toBeNull();
  });

  it("returns null for malformed JSON", () => {
    expect(parseRunEvent("run.started", "{not json")).toBeNull();
  });
});

describe("nodeStatusMap", () => {
  it("projects each node's status by id", () => {
    const state = fold(runStarted, nodeStarted(2, "n1"), nodeStarted(3, "n2"));
    expect(nodeStatusMap(state)).toEqual({ n1: "running", n2: "running" });
  });
});
