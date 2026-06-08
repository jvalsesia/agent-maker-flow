import { afterEach, describe, expect, it, vi } from "vitest";
import { createElement, type ReactNode } from "react";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

vi.mock("./apiClient", () => ({ apiGet: vi.fn() }));

import { apiGet } from "./apiClient";
import { useModels, useProviders } from "./models";

const mockedApiGet = vi.mocked(apiGet);

function wrapper() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return ({ children }: { children: ReactNode }) =>
    createElement(QueryClientProvider, { client }, children);
}

afterEach(() => {
  vi.clearAllMocks();
});

describe("useProviders", () => {
  it("requests the providers path and returns the provider list", async () => {
    mockedApiGet.mockResolvedValue({ providers: [{ id: "openai", model_count: 2 }] });

    const { result } = renderHook(() => useProviders(), { wrapper: wrapper() });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApiGet).toHaveBeenCalledWith("/providers");
    expect(result.current.data).toEqual([{ id: "openai", model_count: 2 }]);
  });
});

describe("useModels", () => {
  it("requests the provider-scoped models path", async () => {
    mockedApiGet.mockResolvedValue({
      provider: "openai",
      models: [{ id: "gpt-4o", label: "gpt-4o", mode: "chat" }],
    });

    const { result } = renderHook(() => useModels("openai"), { wrapper: wrapper() });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedApiGet).toHaveBeenCalledWith("/providers/openai/models");
    expect(result.current.data).toEqual([{ id: "gpt-4o", label: "gpt-4o", mode: "chat" }]);
  });

  it("is disabled until a provider is selected", () => {
    const { result } = renderHook(() => useModels(null), { wrapper: wrapper() });

    expect(result.current.fetchStatus).toBe("idle");
    expect(mockedApiGet).not.toHaveBeenCalled();
  });
});
