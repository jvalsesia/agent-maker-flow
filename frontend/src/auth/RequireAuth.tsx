import { useAuth } from "@clerk/clerk-react";
import type { ReactNode } from "react";
import { Navigate } from "react-router-dom";

/**
 * Route guard: redirects unauthenticated visitors to the sign-in page and
 * renders its children once Clerk confirms a signed-in session. While Clerk is
 * still loading the auth state, renders nothing to avoid a flash of either the
 * protected content or the redirect.
 */
export function RequireAuth({ children }: { children: ReactNode }) {
  const { isLoaded, isSignedIn } = useAuth();

  if (!isLoaded) return null;
  if (!isSignedIn) return <Navigate to="/sign-in" replace />;

  return <>{children}</>;
}
