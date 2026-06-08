import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";

import { RequireAuth } from "./RequireAuth";

const mockUseAuth = vi.fn();
vi.mock("@clerk/clerk-react", () => ({
  useAuth: () => mockUseAuth(),
}));

function renderGuard() {
  return render(
    <MemoryRouter initialEntries={["/"]}>
      <Routes>
        <Route
          path="/"
          element={
            <RequireAuth>
              <div>protected content</div>
            </RequireAuth>
          }
        />
        <Route path="/sign-in" element={<div>sign in screen</div>} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("RequireAuth", () => {
  it("redirects to sign-in when signed out", () => {
    mockUseAuth.mockReturnValue({ isLoaded: true, isSignedIn: false });
    renderGuard();

    expect(screen.getByText("sign in screen")).toBeInTheDocument();
    expect(screen.queryByText("protected content")).not.toBeInTheDocument();
  });

  it("renders children when signed in", () => {
    mockUseAuth.mockReturnValue({ isLoaded: true, isSignedIn: true });
    renderGuard();

    expect(screen.getByText("protected content")).toBeInTheDocument();
  });

  it("renders nothing while Clerk is still loading", () => {
    mockUseAuth.mockReturnValue({ isLoaded: false, isSignedIn: false });
    renderGuard();

    expect(screen.queryByText("protected content")).not.toBeInTheDocument();
    expect(screen.queryByText("sign in screen")).not.toBeInTheDocument();
  });
});
