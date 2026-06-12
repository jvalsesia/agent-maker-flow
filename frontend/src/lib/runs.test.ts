import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock(import("./apiClient"), async (importOriginal) => ({
  ...(await importOriginal()),
  apiGet: vi.fn(),
  apiPost: vi.fn(),
}));

import { apiGet, apiPost, ApiClientError } from "./apiClient";
import { fetchRunSnapshot, startRun, type RunAccepted, type StartRunInput } from "./runs";
import type { FlowGraph } from "./flowGraph";

const mockedApiPost = vi.mocked(apiPost);
const mockedApiGet = vi.mocked(apiGet);

const graph: FlowGraph = {
  nodes: [
    { id: "n1", type: "agent", position: { x: 0, y: 0 }, data: { agentId: "a1" } },
    { id: "n2", type: "agent", position: { x: 240, y: 0 }, data: { agentId: "a2" } },
  ],
  edges: [{ id: "e1", source: "n1", target: "n2" }],
  rootNodeId: "n1",
};

afterEach(() => {
  vi.clearAllMocks();
});

describe("startRun", () => {
  it("POSTs the prompt, graph, and flowId and returns the accepted run", async () => {
    const accepted: RunAccepted = { runId: "run-1", status: "running", nodeCount: 2 };
    mockedApiPost.mockResolvedValue(accepted);
    const input: StartRunInput = { prompt: "Summarize", graph, flowId: "flow-1" };

    const result = await startRun(input);

    expect(mockedApiPost).toHaveBeenCalledWith("/runs", input);
    expect(result).toEqual(accepted);
  });

  it("surfaces the rejection code from the gateway as an ApiClientError", async () => {
    mockedApiPost.mockRejectedValue(
      new ApiClientError({ code: "RUN002", message: "A run is already in progress for this flow." }),
    );

    await expect(startRun({ prompt: "go", graph })).rejects.toMatchObject({
      code: "RUN002",
    });
  });
});

describe("fetchRunSnapshot", () => {
  it("GETs the run path and returns the snapshot", async () => {
    const snapshot = { runId: "run-1", status: "succeeded" as const, output: "done", events: [] };
    mockedApiGet.mockResolvedValue(snapshot);

    const result = await fetchRunSnapshot("run-1");

    expect(mockedApiGet).toHaveBeenCalledWith("/runs/run-1");
    expect(result).toEqual(snapshot);
  });
});
