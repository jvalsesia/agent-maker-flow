import { afterEach, describe, expect, it, vi } from "vitest";
import { createElement, type ReactNode } from "react";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

vi.mock("./apiClient", () => ({
  apiGet: vi.fn(),
  apiPost: vi.fn(),
  apiPut: vi.fn(),
  apiPatch: vi.fn(),
  apiDelete: vi.fn(),
}));

import { apiDelete, apiGet, apiPatch, apiPost, apiPut } from "./apiClient";
import {
  useCreateFlow,
  useDeleteFlow,
  useFlow,
  useFlows,
  useRenameFlow,
  useUpdateFlow,
  type Flow,
  type FlowInput,
  type FlowSummary,
} from "./flows";

const mockedApiGet = vi.mocked(apiGet);
const mockedApiPost = vi.mocked(apiPost);
const mockedApiPut = vi.mocked(apiPut);
const mockedApiPatch = vi.mocked(apiPatch);
const mockedApiDelete = vi.mocked(apiDelete);

/** A wrapper exposing its client so tests can assert query invalidation. */
function makeWrapper() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const invalidateSpy = vi.spyOn(client, "invalidateQueries");
  const Wrapper = ({ children }: { children: ReactNode }) =>
    createElement(QueryClientProvider, { client }, children);
  return { Wrapper, invalidateSpy };
}

const sampleSummary: FlowSummary = {
  id: "flow-1",
  name: "Research pipeline",
  created_at: "2026-06-10T14:00:00Z",
  updated_at: "2026-06-10T14:05:00Z",
};

const sampleFlow: Flow = {
  ...sampleSummary,
  graph: {
    nodes: [
      {
        id: "node-1",
        type: "agent",
        position: { x: 120, y: 80 },
        data: { agentId: "agent-1" },
      },
    ],
    edges: [],
    rootNodeId: "node-1",
  },
};

const sampleInput: FlowInput = {
  name: sampleFlow.name,
  graph: sampleFlow.graph,
};

afterEach(() => {
  vi.clearAllMocks();
});

describe("useFlows", () => {
  it("requests the flows path and returns the summaries", async () => {
    mockedApiGet.mockResolvedValue({ flows: [sampleSummary] });
    const { Wrapper } = makeWrapper();

    const { result } = renderHook(() => useFlows(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApiGet).toHaveBeenCalledWith("/flows");
    expect(result.current.data).toEqual([sampleSummary]);
  });
});

describe("useFlow", () => {
  it("opens a single flow with its graph when an id is given", async () => {
    mockedApiGet.mockResolvedValue(sampleFlow);
    const { Wrapper } = makeWrapper();

    const { result } = renderHook(() => useFlow("flow-1"), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApiGet).toHaveBeenCalledWith("/flows/flow-1");
    expect(result.current.data).toEqual(sampleFlow);
  });

  it("is disabled (no request) when id is null", () => {
    const { Wrapper } = makeWrapper();

    const { result } = renderHook(() => useFlow(null), { wrapper: Wrapper });

    expect(mockedApiGet).not.toHaveBeenCalled();
    expect(result.current.fetchStatus).toBe("idle");
  });
});

describe("useCreateFlow", () => {
  it("POSTs the body and invalidates the flows query", async () => {
    mockedApiPost.mockResolvedValue(sampleFlow);
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useCreateFlow(), { wrapper: Wrapper });
    await result.current.mutateAsync(sampleInput);

    expect(mockedApiPost).toHaveBeenCalledWith("/flows", sampleInput);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["flows"] });
  });
});

describe("useUpdateFlow", () => {
  it("PUTs to the flow path and invalidates the flows query", async () => {
    mockedApiPut.mockResolvedValue(sampleFlow);
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useUpdateFlow(), { wrapper: Wrapper });
    await result.current.mutateAsync({ id: "flow-1", input: sampleInput });

    expect(mockedApiPut).toHaveBeenCalledWith("/flows/flow-1", sampleInput);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["flows"] });
  });
});

describe("useRenameFlow", () => {
  it("PATCHes only the name and invalidates the flows query", async () => {
    mockedApiPatch.mockResolvedValue({ ...sampleSummary, name: "Renamed" });
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useRenameFlow(), { wrapper: Wrapper });
    await result.current.mutateAsync({ id: "flow-1", name: "Renamed" });

    expect(mockedApiPatch).toHaveBeenCalledWith("/flows/flow-1", { name: "Renamed" });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["flows"] });
  });
});

describe("useDeleteFlow", () => {
  it("DELETEs the flow path and invalidates the flows query", async () => {
    mockedApiDelete.mockResolvedValue({ id: "flow-1" });
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useDeleteFlow(), { wrapper: Wrapper });
    await result.current.mutateAsync("flow-1");

    expect(mockedApiDelete).toHaveBeenCalledWith("/flows/flow-1");
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["flows"] });
  });
});
