import { ClerkProvider } from "@clerk/clerk-react";
import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "react-router-dom";

import "./index.css";

import { AuthTokenBridge } from "./auth/AuthTokenBridge";
import { ThemeProvider } from "./hooks/useTheme";
import { applyTheme, resolveInitialTheme } from "./lib/theme";
import { queryClient } from "./lib/queryClient";
import { router } from "./routes/router";

// Apply the resolved theme before first paint to avoid a flash of the wrong
// palette; ThemeProvider keeps it in sync from here on (F11).
applyTheme(resolveInitialTheme());

const PUBLISHABLE_KEY = import.meta.env.VITE_CLERK_PUBLISHABLE_KEY as string | undefined;
if (!PUBLISHABLE_KEY) {
  throw new Error("Missing VITE_CLERK_PUBLISHABLE_KEY environment variable");
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ClerkProvider publishableKey={PUBLISHABLE_KEY}>
      <QueryClientProvider client={queryClient}>
        <ThemeProvider>
          <AuthTokenBridge />
          <RouterProvider router={router} />
        </ThemeProvider>
      </QueryClientProvider>
    </ClerkProvider>
  </React.StrictMode>,
);
