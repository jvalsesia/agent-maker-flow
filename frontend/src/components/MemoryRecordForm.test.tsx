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

const useAgentsMock = vi.fn();
vi.mock("../lib/agents", () => ({
  useAgents: () => useAgentsMock(),
}));

import { useCreateMemoryRecord, useUpdateMemoryRecord, MEMORY_TEXT_MAX } from "../lib/memory";
import { MemoryRecordForm } from "./MemoryRecordForm";

const sampleAgents = [
  { id: "agent-1", name: "Summarizer" },
  { id: "agent-2", name: "Researcher" },
];

const idle = {
  mutateAsync: vi.fn().mockResolvedValue({}),
  isPending: false,
  isSuccess: false,
  error: null,
};

beforeEach(() => {
  vi.mocked(useCreateMemoryRecord).mockReturnValue(idle as never);
  vi.mocked(useUpdateMemoryRecord).mockReturnValue(idle as never);
  useAgentsMock.mockReturnValue({ data: sampleAgents });
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

  it("defaults to global scope and lists the user's agents", () => {
    render(<MemoryRecordForm />);
    const select = screen.getByLabelText("Scope to agent") as HTMLSelectElement;
    expect(select.value).toBe("");
    expect(screen.getByRole("option", { name: "All agents (global)" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Summarizer" })).toBeInTheDocument();
  });

  it("submits the selected agent scope when creating a record", async () => {
    const mutateAsync = vi.fn().mockResolvedValue({});
    vi.mocked(useCreateMemoryRecord).mockReturnValue({ ...idle, mutateAsync } as never);
    render(<MemoryRecordForm />);

    fireEvent.change(screen.getByLabelText("Memory text"), { target: { value: "scoped note" } });
    fireEvent.change(screen.getByLabelText("Scope to agent"), { target: { value: "agent-2" } });
    fireEvent.click(screen.getByRole("button", { name: "Add record" }));

    expect(mutateAsync).toHaveBeenCalledWith({ text: "scoped note", agentId: "agent-2" });
  });

  it("sends null scope for a global record", async () => {
    const mutateAsync = vi.fn().mockResolvedValue({});
    vi.mocked(useCreateMemoryRecord).mockReturnValue({ ...idle, mutateAsync } as never);
    render(<MemoryRecordForm />);

    fireEvent.change(screen.getByLabelText("Memory text"), { target: { value: "global note" } });
    fireEvent.click(screen.getByRole("button", { name: "Add record" }));

    expect(mutateAsync).toHaveBeenCalledWith({ text: "global note", agentId: null });
  });
});
