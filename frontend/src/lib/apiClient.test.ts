import { afterEach, describe, expect, it, vi } from "vitest";

import { apiGet } from "./apiClient";

afterEach(() => {
  vi.restoreAllMocks();
});

function mockJson(body: unknown, status: number) {
  vi.stubGlobal(
    "fetch",
    vi.fn(
      async () =>
        new Response(JSON.stringify(body), {
          status,
          headers: { "Content-Type": "application/json" },
        }),
    ),
  );
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
});
