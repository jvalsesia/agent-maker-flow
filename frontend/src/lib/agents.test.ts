import { afterEach, describe, expect, it, vi } from "vitest";
import { createElement, type ReactNode } from "react";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

vi.mock("./apiClient", () => ({
  apiGet: vi.fn(),
  apiPost: vi.fn(),
  apiPut: vi.fn(),
  apiDelete: vi.fn(),
}));

import { apiDelete, apiGet, apiPost, apiPut } from "./apiClient";
import { useAgents, useCreateAgent, useDeleteAgent, useUpdateAgent, type Agent } from "./agents";

const mockedApiGet = vi.mocked(apiGet);
const mockedApiPost = vi.mocked(apiPost);
const mockedApiPut = vi.mocked(apiPut);
const mockedApiDelete = vi.mocked(apiDelete);

/** A wrapper exposing its client so tests can assert query invalidation. */
function makeWrapper() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const invalidateSpy = vi.spyOn(client, "invalidateQueries");
  const Wrapper = ({ children }: { children: ReactNode }) =>
    createElement(QueryClientProvider, { client }, children);
  return { Wrapper, invalidateSpy };
}

const sampleAgent: Agent = {
  id: "agent-1",
  name: "Summarizer",
  preamble: null,
  system_prompt: "Summarize the input.",
  provider: "openai",
  model: "gpt-4o",
  recent_n: 10,
  top_k: 5,
  created_at: "2026-06-09T00:00:00Z",
  updated_at: "2026-06-09T00:00:00Z",
};

const sampleInput = {
  name: "Summarizer",
  preamble: null,
  system_prompt: "Summarize the input.",
  provider: "openai",
  model: "gpt-4o",
  recent_n: 10,
  top_k: 5,
};

afterEach(() => {
  vi.clearAllMocks();
});

describe("useAgents", () => {
  it("requests the agents path and returns the list", async () => {
    mockedApiGet.mockResolvedValue({ agents: [sampleAgent] });
    const { Wrapper } = makeWrapper();

    const { result } = renderHook(() => useAgents(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApiGet).toHaveBeenCalledWith("/agents");
    expect(result.current.data).toEqual([sampleAgent]);
  });
});

describe("useCreateAgent", () => {
  it("POSTs the body and invalidates the agents query", async () => {
    mockedApiPost.mockResolvedValue(sampleAgent);
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useCreateAgent(), { wrapper: Wrapper });
    await result.current.mutateAsync(sampleInput);

    expect(mockedApiPost).toHaveBeenCalledWith("/agents", sampleInput);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["agents"] });
  });
});

describe("useUpdateAgent", () => {
  it("PUTs to the agent path and invalidates the agents query", async () => {
    mockedApiPut.mockResolvedValue({ ...sampleAgent, name: "Renamed" });
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useUpdateAgent(), { wrapper: Wrapper });
    await result.current.mutateAsync({ id: "agent-1", input: sampleInput });

    expect(mockedApiPut).toHaveBeenCalledWith("/agents/agent-1", sampleInput);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["agents"] });
  });
});

describe("useDeleteAgent", () => {
  it("DELETEs the agent path and invalidates the agents query", async () => {
    mockedApiDelete.mockResolvedValue({ id: "agent-1" });
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useDeleteAgent(), { wrapper: Wrapper });
    await result.current.mutateAsync("agent-1");

    expect(mockedApiDelete).toHaveBeenCalledWith("/agents/agent-1");
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["agents"] });
  });
});
