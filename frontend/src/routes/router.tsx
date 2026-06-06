import { createBrowserRouter, Navigate, type RouteObject } from "react-router-dom";

import { AppShell } from "../components/AppShell";
import { AgentsPage } from "../pages/AgentsPage";
import { FlowsPage } from "../pages/FlowsPage";

/** Route tree: layout shell with Agents and Flows children. */
export const routes: RouteObject[] = [
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <Navigate to="/agents" replace /> },
      { path: "agents", element: <AgentsPage /> },
      { path: "flows", element: <FlowsPage /> },
    ],
  },
];

export const router = createBrowserRouter(routes);
