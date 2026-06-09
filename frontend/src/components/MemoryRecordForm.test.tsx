import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";

vi.mock("../lib/memory", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../lib/memory")>();
  return {
    ...actual,
    useCreateMemoryRecord: vi.fn(),
    useUpdateMemoryRecord: vi.fn(),
  };
});

import { useCreateMemoryRecord, useUpdateMemoryRecord, MEMORY_TEXT_MAX } from "../lib/memory";
import { MemoryRecordForm } from "./MemoryRecordForm";

const idle = {
  mutateAsync: vi.fn().mockResolvedValue({}),
  isPending: false,
  isSuccess: false,
  error: null,
};

beforeEach(() => {
  vi.mocked(useCreateMemoryRecord).mockReturnValue(idle as never);
  vi.mocked(useUpdateMemoryRecord).mockReturnValue(idle as never);
});

afterEach(() => {
  vi.clearAllMocks();
});

describe("MemoryRecordForm", () => {
  it("disables submit and shows the counter for empty input", () => {
    render(<MemoryRecordForm />);
    expect(screen.getByLabelText("character count")).toHaveTextContent(`0 / ${MEMORY_TEXT_MAX}`);
    expect(screen.getByRole("button", { name: "Add record" })).toBeDisabled();
  });

  it("tracks the character count as text changes", () => {
    render(<MemoryRecordForm />);
    fireEvent.change(screen.getByLabelText("Memory text"), { target: { value: "hello" } });
    expect(screen.getByLabelText("character count")).toHaveTextContent(`5 / ${MEMORY_TEXT_MAX}`);
    expect(screen.getByRole("button", { name: "Add record" })).toBeEnabled();
  });

  it("blocks submit and warns when over the size limit", () => {
    render(<MemoryRecordForm />);
    fireEvent.change(screen.getByLabelText("Memory text"), {
      target: { value: "a".repeat(MEMORY_TEXT_MAX + 1) },
    });
    expect(screen.getByRole("alert")).toHaveTextContent("8000 characters or fewer");
    expect(screen.getByRole("button", { name: "Add record" })).toBeDisabled();
  });

  it("reflects the embedding-in-progress state", () => {
    vi.mocked(useCreateMemoryRecord).mockReturnValue({ ...idle, isPending: true } as never);
    render(<MemoryRecordForm />);
    expect(screen.getByRole("status")).toHaveTextContent("Embedding…");
    expect(screen.getByRole("button", { name: "Embedding…" })).toBeDisabled();
  });
});
