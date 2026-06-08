import { createBrowserRouter, Navigate, type RouteObject } from "react-router-dom";

import { RequireAuth } from "../auth/RequireAuth";
import { AppShell } from "../components/AppShell";
import { AgentsPage } from "../pages/AgentsPage";
import { FlowsPage } from "../pages/FlowsPage";
import { SignInPage } from "../pages/SignInPage";
import { SignUpPage } from "../pages/SignUpPage";

/**
 * Route tree: public Clerk sign-in/up routes, plus the protected layout shell
 * (Agents and Flows) guarded by `RequireAuth`. The shell's index redirects to
 * `/agents`, so a signed-in user lands on the Agents Dashboard.
 */
export const routes: RouteObject[] = [
  { path: "/sign-in/*", element: <SignInPage /> },
  { path: "/sign-up/*", element: <SignUpPage /> },
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
    ],
  },
];

export const router = createBrowserRouter(routes);
