import { useAuth } from "@clerk/clerk-react";
import { useEffect } from "react";

import { setTokenGetter } from "../lib/authToken";

/**
 * Registers Clerk's `getToken` into the module-level token registry so the
 * non-hook API client and SSE helpers can attach the current session token.
 * Renders nothing; mount once inside the Clerk provider.
 */
export function AuthTokenBridge() {
  const { getToken } = useAuth();

  useEffect(() => {
    setTokenGetter(() => getToken());
    return () => setTokenGetter(null);
  }, [getToken]);

  return null;
}
