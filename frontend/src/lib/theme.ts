/** Theme persistence + resolution (F11). Dark ships as the default: a stored
 * choice wins, otherwise we honour the OS preference, otherwise dark. */

export type Theme = "light" | "dark";

const STORAGE_KEY = "amf-theme";

export function readStoredTheme(): Theme | null {
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    return value === "light" || value === "dark" ? value : null;
  } catch {
    return null;
  }
}

export function storeTheme(theme: Theme): void {
  try {
    localStorage.setItem(STORAGE_KEY, theme);
  } catch {
    /* private mode / disabled storage — non-fatal */
  }
}

/** Stored choice wins; else OS preference; else dark (ship-dark default). */
export function resolveInitialTheme(): Theme {
  const stored = readStoredTheme();
  if (stored) return stored;
  if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  }
  return "dark";
}

export function applyTheme(theme: Theme): void {
  document.documentElement.setAttribute("data-theme", theme);
}
