import { lazy, Suspense, type ReactNode } from "react";
import { createBrowserRouter, Navigate, type RouteObject } from "react-router-dom";

import { RequireAuth } from "../auth/RequireAuth";
import { AppShell } from "../components/AppShell";
import { Spinner } from "../components/ui";

// Route-level code-splitting: each page is its own chunk so the initial bundle
// stays small. FlowsPage drags in React Flow and the auth pages drag in the
// Clerk UI — both now load only when their route is visited. Pages are named
// exports, hence the `.then(...)` default-mapping for React.lazy.
const AgentsPage = lazy(() =>
  import("../pages/AgentsPage").then((m) => ({ default: m.AgentsPage })),
);
const FlowsPage = lazy(() =>
  import("../pages/FlowsPage").then((m) => ({ default: m.FlowsPage })),
);
const SettingsPage = lazy(() =>
  import("../pages/SettingsPage").then((m) => ({ default: m.SettingsPage })),
);
const SignInPage = lazy(() =>
  import("../pages/SignInPage").then((m) => ({ default: m.SignInPage })),
);
const SignUpPage = lazy(() =>
  import("../pages/SignUpPage").then((m) => ({ default: m.SignUpPage })),
);

/** Suspense boundary for the public (pre-shell) routes; the shell provides its
 *  own boundary around the authenticated outlet. */
function lazyRoute(element: ReactNode): ReactNode {
  return <Suspense fallback={<Spinner label="Loading…" />}>{element}</Suspense>;
}

/**
 * Route tree: public Clerk sign-in/up routes, plus the protected layout shell
 * (Agents and Flows) guarded by `RequireAuth`. The shell's index redirects to
 * `/agents`, so a signed-in user lands on the Agents Dashboard.
 */
export const routes: RouteObject[] = [
  { path: "/sign-in/*", element: lazyRoute(<SignInPage />) },
  { path: "/sign-up/*", element: lazyRoute(<SignUpPage />) },
  {
    path: "/",
    element: (
      <RequireAuth>
        <AppShell />
      </RequireAuth>
    ),
    children: [
      { index: true, element: <Navigate to="/agents" replace /> },
      { path: "agents", element: <AgentsPage /> },
      { path: "flows", element: <FlowsPage /> },
      { path: "settings", element: <SettingsPage /> },
    ],
  },
];

export const router = createBrowserRouter(routes);
