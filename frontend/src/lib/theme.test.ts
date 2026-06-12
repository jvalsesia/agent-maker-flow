import { afterEach, describe, expect, it, vi } from "vitest";

import { applyTheme, readStoredTheme, resolveInitialTheme, storeTheme } from "./theme";

afterEach(() => {
  localStorage.clear();
  document.documentElement.removeAttribute("data-theme");
  vi.unstubAllGlobals();
});

describe("theme persistence", () => {
  it("round-trips a stored theme", () => {
    storeTheme("light");
    expect(readStoredTheme()).toBe("light");
  });

  it("ignores invalid stored values", () => {
    localStorage.setItem("amf-theme", "neon");
    expect(readStoredTheme()).toBeNull();
  });
});

describe("resolveInitialTheme", () => {
  it("prefers a stored choice over OS preference", () => {
    storeTheme("light");
    vi.stubGlobal("matchMedia", () => ({ matches: true }) as MediaQueryList);
    expect(resolveInitialTheme()).toBe("light");
  });

  it("falls back to OS light preference when nothing stored", () => {
    vi.stubGlobal(
      "matchMedia",
      (query: string) => ({ matches: query.includes("light") }) as MediaQueryList,
    );
    expect(resolveInitialTheme()).toBe("light");
  });

  it("defaults to dark when nothing stored and OS prefers dark", () => {
    vi.stubGlobal("matchMedia", () => ({ matches: false }) as MediaQueryList);
    expect(resolveInitialTheme()).toBe("dark");
  });
});

describe("applyTheme", () => {
  it("sets the data-theme attribute on the document root", () => {
    applyTheme("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });
});
