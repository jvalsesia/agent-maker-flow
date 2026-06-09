import { afterEach, describe, expect, it, vi } from "vitest";
import { createElement, type ReactNode } from "react";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

vi.mock("./apiClient", () => ({
  apiGet: vi.fn(),
  apiPut: vi.fn(),
  apiDelete: vi.fn(),
}));

import { apiDelete, apiGet, apiPut } from "./apiClient";
import {
  useDeleteSemanticProfile,
  useEmbeddingSetting,
  useSemanticProfile,
  useSetEmbeddingSetting,
  useSetSemanticProfile,
} from "./embeddingSettings";

const mockedApiGet = vi.mocked(apiGet);
const mockedApiPut = vi.mocked(apiPut);
const mockedApiDelete = vi.mocked(apiDelete);

function makeWrapper() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const invalidateSpy = vi.spyOn(client, "invalidateQueries");
  const Wrapper = ({ children }: { children: ReactNode }) =>
    createElement(QueryClientProvider, { client }, children);
  return { Wrapper, invalidateSpy };
}

afterEach(() => {
  vi.clearAllMocks();
});

describe("useEmbeddingSetting", () => {
  it("GETs the embedding setting path", async () => {
    mockedApiGet.mockResolvedValue({ embedding_model: "text-embedding-3-small" });
    const { Wrapper } = makeWrapper();

    const { result } = renderHook(() => useEmbeddingSetting(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApiGet).toHaveBeenCalledWith("/settings/embedding");
    expect(result.current.data).toEqual({ embedding_model: "text-embedding-3-small" });
  });
});

describe("useSetEmbeddingSetting", () => {
  it("PUTs the model and invalidates the setting query", async () => {
    mockedApiPut.mockResolvedValue({ embedding_model: "text-embedding-3-small" });
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useSetEmbeddingSetting(), { wrapper: Wrapper });
    await result.current.mutateAsync("text-embedding-3-small");

    expect(mockedApiPut).toHaveBeenCalledWith("/settings/embedding", {
      embedding_model: "text-embedding-3-small",
    });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["embedding-setting"] });
  });
});

describe("semantic profile hooks", () => {
  it("GETs the profile for an agent", async () => {
    mockedApiGet.mockResolvedValue({
      agent_id: "a1",
      embedding_model: "m",
      memory_scope: "all",
    });
    const { Wrapper } = makeWrapper();

    const { result } = renderHook(() => useSemanticProfile("a1"), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApiGet).toHaveBeenCalledWith("/agents/a1/semantic-profile");
  });

  it("is disabled when no agent id is given", () => {
    const { Wrapper } = makeWrapper();
    const { result } = renderHook(() => useSemanticProfile(null), { wrapper: Wrapper });
    expect(result.current.fetchStatus).toBe("idle");
    expect(mockedApiGet).not.toHaveBeenCalled();
  });

  it("PUTs the profile and invalidates it", async () => {
    mockedApiPut.mockResolvedValue({ agent_id: "a1", embedding_model: "m", memory_scope: "own" });
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useSetSemanticProfile("a1"), { wrapper: Wrapper });
    await result.current.mutateAsync({ embedding_model: "m", memory_scope: "own" });

    expect(mockedApiPut).toHaveBeenCalledWith("/agents/a1/semantic-profile", {
      embedding_model: "m",
      memory_scope: "own",
    });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["semantic-profile", "a1"] });
  });

  it("DELETEs the profile and invalidates it", async () => {
    mockedApiDelete.mockResolvedValue({ agent_id: "a1" });
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useDeleteSemanticProfile("a1"), { wrapper: Wrapper });
    await result.current.mutateAsync();

    expect(mockedApiDelete).toHaveBeenCalledWith("/agents/a1/semantic-profile");
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["semantic-profile", "a1"] });
  });
});
