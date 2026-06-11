import { afterEach, describe, expect, it, vi } from "vitest";

import { apiGet, apiPatch } from "./apiClient";
import { setTokenGetter } from "./authToken";

afterEach(() => {
  vi.restoreAllMocks();
  setTokenGetter(null);
});

function mockJson(body: unknown, status: number) {
  const fetchMock = vi.fn(
    async (_input: RequestInfo | URL, _init?: RequestInit) =>
      new Response(JSON.stringify(body), {
        status,
        headers: { "Content-Type": "application/json" },
      }),
  );
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

describe("apiClient", () => {
  it("parses the success envelope and returns data", async () => {
    mockJson({ status: "success", data: { service: "up" } }, 200);

    const data = await apiGet<{ service: string }>("/health");

    expect(data).toEqual({ service: "up" });
  });

  it("maps the error envelope to an ApiClientError with code and message", async () => {
    mockJson({ status: "error", error: { code: "HEALTH001", message: "cache down" } }, 503);

    await expect(apiGet("/health")).rejects.toMatchObject({
      code: "HEALTH001",
      message: "cache down",
    });
  });

  it("attaches the Bearer token when a session token is present", async () => {
    const fetchMock = mockJson({ status: "success", data: { user_id: "user_1" } }, 200);
    setTokenGetter(async () => "jwt-abc");

    await apiGet("/me");

    const init = fetchMock.mock.calls[0][1] as RequestInit;
    const headers = init.headers as Record<string, string>;
    expect(headers.Authorization).toBe("Bearer jwt-abc");
  });

  it("omits the Authorization header when no token is available", async () => {
    const fetchMock = mockJson({ status: "success", data: {} }, 200);
    setTokenGetter(async () => null);

    await apiGet("/health");

    const init = fetchMock.mock.calls[0][1] as RequestInit;
    const headers = init.headers as Record<string, string>;
    expect(headers.Authorization).toBeUndefined();
  });

  it("maps a 401 AUTH001 response to an ApiClientError", async () => {
    mockJson(
      { status: "error", error: { code: "AUTH001", message: "Session expired or invalid." } },
      401,
    );

    await expect(apiGet("/me")).rejects.toMatchObject({ code: "AUTH001" });
  });

  it("sends a PATCH with the serialized body and returns data", async () => {
    const fetchMock = mockJson({ status: "success", data: { id: "flow-1", name: "Renamed" } }, 200);

    const data = await apiPatch<{ id: string; name: string }>("/flows/flow-1", {
      name: "Renamed",
    });

    expect(data).toEqual({ id: "flow-1", name: "Renamed" });
    const init = fetchMock.mock.calls[0][1] as RequestInit;
    expect(init.method).toBe("PATCH");
    expect(init.body).toBe(JSON.stringify({ name: "Renamed" }));
  });
});
