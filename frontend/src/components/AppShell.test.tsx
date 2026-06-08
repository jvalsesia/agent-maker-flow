import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, useRoutes } from "react-router-dom";

// The route tree is guarded by RequireAuth and renders Clerk components; mock
// the Clerk module to a signed-in session so the shell renders under test.
vi.mock("@clerk/clerk-react", () => ({
  useAuth: () => ({ isLoaded: true, isSignedIn: true }),
  UserButton: () => null,
  SignIn: () => null,
  SignUp: () => null,
}));

import { routes } from "../routes/router";

function Routed() {
  return useRoutes(routes);
}

function renderApp(initialEntries: string[]) {
  return render(
    <MemoryRouter initialEntries={initialEntries}>
      <Routed />
    </MemoryRouter>,
  );
}

describe("AppShell", () => {
  it("renders navigation and defaults to the agents route", () => {
    renderApp(["/"]);

    expect(screen.getByRole("link", { name: "Agents" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Flows" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Agents" })).toBeInTheDocument();
  });

  it("navigates between routes without a full reload", async () => {
    const user = userEvent.setup();
    renderApp(["/agents"]);

    await user.click(screen.getByRole("link", { name: "Flows" }));
    expect(screen.getByRole("heading", { name: "Flows" })).toBeInTheDocument();

    await user.click(screen.getByRole("link", { name: "Agents" }));
    expect(screen.getByRole("heading", { name: "Agents" })).toBeInTheDocument();
  });
});
