/**
 * REST client for the backend. Parses the platform-wide success/error
 * envelopes into typed results, throwing a typed error on failure.
 */

export interface ApiError {
  code: string;
  message: string;
}

export class ApiClientError extends Error {
  code: string;

  constructor(error: ApiError) {
    super(error.message);
    this.name = "ApiClientError";
    this.code = error.code;
  }
}

interface SuccessEnvelope<T> {
  status: "success";
  data: T;
}

interface ErrorEnvelope {
  status: "error";
  error: ApiError;
}

const BASE_URL = "/api/v1";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    headers: { "Content-Type": "application/json", ...(init?.headers ?? {}) },
    ...init,
  });

  const body = (await res.json()) as SuccessEnvelope<T> | ErrorEnvelope;

  if (!res.ok || body.status === "error") {
    const error =
      (body as ErrorEnvelope).error ??
      ({ code: "UNKNOWN", message: `Request failed (${res.status})` } satisfies ApiError);
    throw new ApiClientError(error);
  }

  return (body as SuccessEnvelope<T>).data;
}

export function apiGet<T>(path: string, init?: RequestInit): Promise<T> {
  return request<T>(path, { method: "GET", ...init });
}
