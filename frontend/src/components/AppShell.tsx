import { Outlet } from "react-router-dom";

import { NavBar } from "./NavBar";

/** Persistent layout: header with navigation and a routed content outlet. */
export function AppShell() {
  return (
    <div className="app-shell">
      <header>
        <strong>Agent Maker Flow</strong>
        <NavBar />
      </header>
      <main>
        <Outlet />
      </main>
    </div>
  );
}
