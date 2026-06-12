import { Outlet, useLocation } from "react-router-dom";

import { NavBar } from "./NavBar";
import { ToastProvider } from "./ui";
import styles from "./AppShell.module.css";

/** Persistent layout: a sticky top bar with brand + navigation, and a routed
 * content outlet. The Flows workspace runs full-bleed (its own full-height
 * three-pane layout), so the centered page padding is dropped there. */
export function AppShell() {
  const { pathname } = useLocation();
  const flush = pathname.startsWith("/flows");

  return (
    <ToastProvider>
      <div className={styles.shell}>
        <a className="skip-link" href="#main">
          Skip to content
        </a>
        <header className={styles.topbar}>
          <span className={styles.brand}>
            <svg
              className={styles.brandGlyph}
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              aria-hidden="true"
            >
              <circle cx="6" cy="6" r="2.5" />
              <circle cx="18" cy="12" r="2.5" />
              <circle cx="6" cy="18" r="2.5" />
              <path strokeLinecap="round" d="M8.2 7.1l7.6 3.8M8.2 16.9l7.6-3.8" />
            </svg>
            Agent Maker Flow
          </span>
          <NavBar />
        </header>
        <main
          id="main"
          className={[styles.main, flush && styles.mainFlush].filter(Boolean).join(" ")}
        >
          <Outlet />
        </main>
      </div>
    </ToastProvider>
  );
}
