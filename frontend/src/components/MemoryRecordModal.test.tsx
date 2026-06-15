import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

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

import {
  useCreateMemoryRecord,
  useUpdateMemoryRecord,
  MEMORY_TEXT_MAX,
  type MemoryRecord,
} from "../lib/memory";
import { MemoryRecordModal } from "./MemoryRecordModal";

const sampleAgents = [
  { id: "agent-1", name: "Summarizer" },
  { id: "agent-2", name: "Researcher" },
];

const sampleRecord: MemoryRecord = {
  id: "rec-1",
  text: "The ideal customer profile is B2B SaaS, 50-500 employees.",
  embedding_model: "text-embedding-3-small",
  agent_id: "agent-1",
  char_count: 56,
  created_at: "2026-06-14T16:32:00.000Z",
  updated_at: "2026-06-14T16:32:00.000Z",
};

const idle = {
  mutateAsync: vi.fn().mockResolvedValue({}),
  isPending: false,
  isSuccess: false,
  error: null,
};

const noop = () => {};

beforeEach(() => {
  vi.mocked(useCreateMemoryRecord).mockReturnValue(idle as never);
  vi.mocked(useUpdateMemoryRecord).mockReturnValue(idle as never);
  useAgentsMock.mockReturnValue({ data: sampleAgents });
});

afterEach(() => {
  vi.clearAllMocks();
});

describe("MemoryRecordModal — create mode", () => {
  it("disables submit and shows the counter for empty input", () => {
    render(<MemoryRecordModal mode="create" onClose={noop} />);
    expect(screen.getByLabelText("character count")).toHaveTextContent(`0 / ${MEMORY_TEXT_MAX}`);
    expect(screen.getByRole("button", { name: "Add record" })).toBeDisabled();
  });

  it("tracks the character count as text changes", () => {
    render(<MemoryRecordModal mode="create" onClose={noop} />);
    fireEvent.change(screen.getByLabelText("Memory text"), { target: { value: "hello" } });
    expect(screen.getByLabelText("character count")).toHaveTextContent(`5 / ${MEMORY_TEXT_MAX}`);
    expect(screen.getByRole("button", { name: "Add record" })).toBeEnabled();
  });

  it("blocks submit and warns when over the size limit", () => {
    render(<MemoryRecordModal mode="create" onClose={noop} />);
    fireEvent.change(screen.getByLabelText("Memory text"), {
      target: { value: "a".repeat(MEMORY_TEXT_MAX + 1) },
    });
    expect(screen.getByRole("alert")).toHaveTextContent("8000 characters or fewer");
    expect(screen.getByRole("button", { name: "Add record" })).toBeDisabled();
  });

  it("reflects the embedding-in-progress state", () => {
    vi.mocked(useCreateMemoryRecord).mockReturnValue({ ...idle, isPending: true } as never);
    render(<MemoryRecordModal mode="create" onClose={noop} />);
    expect(screen.getByRole("status")).toHaveTextContent("Embedding…");
    expect(screen.getByRole("button", { name: "Embedding…" })).toBeDisabled();
  });

  it("defaults to global scope and lists the user's agents", () => {
    render(<MemoryRecordModal mode="create" onClose={noop} />);
    const select = screen.getByLabelText("Scope to agent") as HTMLSelectElement;
    expect(select.value).toBe("");
    expect(screen.getByRole("option", { name: "All agents (global)" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Summarizer" })).toBeInTheDocument();
  });

  it("submits the selected agent scope and notifies onSaved", async () => {
    const user = userEvent.setup();
    const mutateAsync = vi.fn().mockResolvedValue({});
    const onSaved = vi.fn();
    vi.mocked(useCreateMemoryRecord).mockReturnValue({ ...idle, mutateAsync } as never);
    render(<MemoryRecordModal mode="create" onClose={noop} onSaved={onSaved} />);

    fireEvent.change(screen.getByLabelText("Memory text"), { target: { value: "scoped note" } });
    fireEvent.change(screen.getByLabelText("Scope to agent"), { target: { value: "agent-2" } });
    await user.click(screen.getByRole("button", { name: "Add record" }));

    expect(mutateAsync).toHaveBeenCalledWith({ text: "scoped note", agentId: "agent-2" });
    expect(onSaved).toHaveBeenCalledTimes(1);
  });

  it("sends null scope for a global record", async () => {
    const user = userEvent.setup();
    const mutateAsync = vi.fn().mockResolvedValue({});
    vi.mocked(useCreateMemoryRecord).mockReturnValue({ ...idle, mutateAsync } as never);
    render(<MemoryRecordModal mode="create" onClose={noop} />);

    fireEvent.change(screen.getByLabelText("Memory text"), { target: { value: "global note" } });
    await user.click(screen.getByRole("button", { name: "Add record" }));

    expect(mutateAsync).toHaveBeenCalledWith({ text: "global note", agentId: null });
  });
});

describe("MemoryRecordModal — edit mode", () => {
  it("prefills the record and updates it by id", async () => {
    const user = userEvent.setup();
    const mutateAsync = vi.fn().mockResolvedValue({});
    vi.mocked(useUpdateMemoryRecord).mockReturnValue({ ...idle, mutateAsync } as never);
    render(<MemoryRecordModal mode="edit" record={sampleRecord} onClose={noop} />);

    const textarea = screen.getByLabelText("Memory text") as HTMLTextAreaElement;
    expect(textarea.value).toBe(sampleRecord.text);
    expect((screen.getByLabelText("Scope to agent") as HTMLSelectElement).value).toBe("agent-1");

    fireEvent.change(textarea, { target: { value: "updated text" } });
    await user.click(screen.getByRole("button", { name: "Save changes" }));

    expect(mutateAsync).toHaveBeenCalledWith({
      id: "rec-1",
      text: "updated text",
      agentId: "agent-1",
    });
  });
});

describe("MemoryRecordModal — view mode", () => {
  it("renders the stored text and metadata read-only", () => {
    render(
      <MemoryRecordModal
        mode="view"
        record={sampleRecord}
        agentNames={{ "agent-1": "Summarizer" }}
        onClose={noop}
      />,
    );

    expect(screen.getByText(sampleRecord.text)).toBeInTheDocument();
    expect(screen.getByText("text-embedding-3-small")).toBeInTheDocument();
    expect(screen.getByText("56 chars")).toBeInTheDocument();
    expect(screen.getByText("Private to Summarizer")).toBeInTheDocument();
    // No editing controls in view mode.
    expect(screen.queryByLabelText("Memory text")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Save changes" })).not.toBeInTheDocument();
  });

  it("invokes onEdit from the Edit action", async () => {
    const user = userEvent.setup();
    const onEdit = vi.fn();
    render(
      <MemoryRecordModal mode="view" record={sampleRecord} onClose={noop} onEdit={onEdit} />,
    );

    await user.click(screen.getByRole("button", { name: "Edit" }));
    expect(onEdit).toHaveBeenCalledTimes(1);
  });
});
