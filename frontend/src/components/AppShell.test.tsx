import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, useRoutes } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

// The route tree is guarded by RequireAuth and renders Clerk components; mock
// the Clerk module to a signed-in session so the shell renders under test.
vi.mock("@clerk/clerk-react", () => ({
  useAuth: () => ({ isLoaded: true, isSignedIn: true }),
  UserButton: () => null,
  SignIn: () => null,
  SignUp: () => null,
}));

import { ThemeProvider } from "../hooks/useTheme";
import { routes } from "../routes/router";

function Routed() {
  return useRoutes(routes);
}

function renderApp(initialEntries: string[]) {
  // The Agents page reads via react-query, so the shell needs a query client;
  // the top-bar theme toggle needs a ThemeProvider.
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <ThemeProvider>
        <MemoryRouter initialEntries={initialEntries}>
          <Routed />
        </MemoryRouter>
      </ThemeProvider>
    </QueryClientProvider>,
  );
}

describe("AppShell", () => {
  it("renders navigation and defaults to the agents route", async () => {
    renderApp(["/"]);

    expect(screen.getByRole("link", { name: "Agents" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Flows" })).toBeInTheDocument();
    // Pages are route-split (React.lazy), so the heading resolves asynchronously.
    expect(await screen.findByRole("heading", { name: "Agents" })).toBeInTheDocument();
  });

  it("navigates between routes without a full reload", async () => {
    const user = userEvent.setup();
    renderApp(["/agents"]);

    await user.click(screen.getByRole("link", { name: "Flows" }));
    expect(await screen.findByRole("heading", { name: "Flows" })).toBeInTheDocument();

    await user.click(screen.getByRole("link", { name: "Agents" }));
    expect(await screen.findByRole("heading", { name: "Agents" })).toBeInTheDocument();
  });
});
