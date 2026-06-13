import type { Appearance } from "@clerk/shared/types";

/**
 * Centralized Clerk appearance for the public auth pages (F11). Mapping Clerk's
 * theme variables onto our design tokens (`var(--…)` from index.css) lets the
 * hosted SignIn/SignUp widgets inherit the app's palette and follow the
 * light/dark theme automatically — no second source of truth. Both
 * `SignInPage` and `SignUpPage` consume this object.
 */
export const clerkAppearance: Appearance = {
  variables: {
    colorPrimary: "var(--accent)",
    colorBackground: "var(--bg-surface-raised)",
    colorText: "var(--text-primary)",
    colorTextSecondary: "var(--text-secondary)",
    colorInputBackground: "var(--bg-base)",
    colorInputText: "var(--text-primary)",
    colorDanger: "var(--danger)",
    colorSuccess: "var(--success)",
    colorWarning: "var(--warning)",
    fontFamily: "var(--font-ui)",
    borderRadius: "var(--radius-md)",
  },
  elements: {
    card: {
      boxShadow: "var(--shadow-lg)",
      border: "1px solid var(--border-subtle)",
    },
    // The widget is centered by AuthLayout; drop Clerk's own outer margin.
    rootBox: { width: "100%" },
  },
};
