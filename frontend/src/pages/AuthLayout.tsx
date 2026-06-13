import type { ReactNode } from "react";

import styles from "./AuthLayout.module.css";

interface AuthLayoutProps {
  /** Accessible label for the page region (e.g. "Sign in"). */
  label: string;
  children: ReactNode;
}

/**
 * Centers a public Clerk auth widget in the viewport (F11). Shared by the
 * sign-in and sign-up pages so both render identically centered and themed.
 */
export function AuthLayout({ label, children }: AuthLayoutProps) {
  return (
    <main className={styles.page} aria-label={label}>
      <div className={styles.content}>{children}</div>
    </main>
  );
}
