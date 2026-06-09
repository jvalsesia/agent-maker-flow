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
import {
  useCreateMemoryRecord,
  useDeleteMemoryRecord,
  useMemoryRecords,
  useUpdateMemoryRecord,
} from "./memory";

const mockedApiGet = vi.mocked(apiGet);
const mockedApiPost = vi.mocked(apiPost);
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

describe("useMemoryRecords", () => {
  it("GETs the memory path and returns records + models in use", async () => {
    mockedApiGet.mockResolvedValue({ records: [], models_in_use: ["m"] });
    const { Wrapper } = makeWrapper();

    const { result } = renderHook(() => useMemoryRecords(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApiGet).toHaveBeenCalledWith("/memory");
    expect(result.current.data).toEqual({ records: [], models_in_use: ["m"] });
  });
});

describe("memory mutations", () => {
  it("POSTs the text and invalidates memory", async () => {
    mockedApiPost.mockResolvedValue({ id: "r1" });
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useCreateMemoryRecord(), { wrapper: Wrapper });
    await result.current.mutateAsync("remember this");

    expect(mockedApiPost).toHaveBeenCalledWith("/memory", { text: "remember this" });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["memory"] });
  });

  it("PUTs to the record path and invalidates memory", async () => {
    mockedApiPut.mockResolvedValue({ id: "r1" });
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useUpdateMemoryRecord(), { wrapper: Wrapper });
    await result.current.mutateAsync({ id: "r1", text: "new text" });

    expect(mockedApiPut).toHaveBeenCalledWith("/memory/r1", { text: "new text" });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["memory"] });
  });

  it("DELETEs the record path and invalidates memory", async () => {
    mockedApiDelete.mockResolvedValue({ id: "r1" });
    const { Wrapper, invalidateSpy } = makeWrapper();

    const { result } = renderHook(() => useDeleteMemoryRecord(), { wrapper: Wrapper });
    await result.current.mutateAsync("r1");

    expect(mockedApiDelete).toHaveBeenCalledWith("/memory/r1");
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["memory"] });
  });
});
