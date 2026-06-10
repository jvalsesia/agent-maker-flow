import "@testing-library/jest-dom/vitest";

// React Flow (@xyflow/react) measures node/pane layout via ResizeObserver,
// which jsdom does not implement. Provide a no-op so canvas components mount.
class ResizeObserverMock {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

if (!("ResizeObserver" in globalThis)) {
  globalThis.ResizeObserver = ResizeObserverMock as unknown as typeof ResizeObserver;
}
