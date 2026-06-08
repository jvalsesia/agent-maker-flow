/**
 * Module-level bridge between Clerk's React hooks and the non-hook API client.
 *
 * `AuthTokenBridge` registers Clerk's `getToken` here via `setTokenGetter`;
 * the API client and SSE helpers read the current token through `getAuthToken`
 * without needing to be inside a React component.
 */

export type TokenGetter = () => Promise<string | null>;

let tokenGetter: TokenGetter | null = null;

/** Register (or clear, with `null`) the active session-token getter. */
export function setTokenGetter(getter: TokenGetter | null): void {
  tokenGetter = getter;
}

/** Resolve the current session token, or `null` when signed out/unavailable. */
export async function getAuthToken(): Promise<string | null> {
  if (!tokenGetter) return null;
  try {
    return await tokenGetter();
  } catch {
    return null;
  }
}

/**
 * Append the current session token as a `token` query parameter to an SSE URL.
 * Native `EventSource` cannot set an Authorization header, so the backend reads
 * the token from the query string for SSE connections. Returns the URL
 * unchanged when no token is available.
 */
export async function sseUrlWithToken(url: string): Promise<string> {
  const token = await getAuthToken();
  if (!token) return url;
  const separator = url.includes("?") ? "&" : "?";
  return `${url}${separator}token=${encodeURIComponent(token)}`;
}
