import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import type { Agent } from "../../lib/agents";

const sampleAgents: Agent[] = [
  {
    id: "agent-1",
    name: "Summarizer",
    preamble: null,
    system_prompt: "Summarize.",
    provider: "openai",
    model: "gpt-4o",
    recent_n: 10,
    top_k: 5,
    created_at: "2026-06-09T00:00:00Z",
    updated_at: "2026-06-09T00:00:00Z",
  },
  {
    id: "agent-2",
    name: "Researcher",
    preamble: null,
    system_prompt: "Research.",
    provider: "anthropic",
    model: "claude-3-5-sonnet",
    recent_n: 20,
    top_k: 8,
    created_at: "2026-06-09T00:00:00Z",
    updated_at: "2026-06-09T00:00:00Z",
  },
];

// Stub the F04 registry hook so the palette renders a deterministic list.
const useAgentsMock = vi.fn();
vi.mock("../../lib/agents", () => ({
  useAgents: () => useAgentsMock(),
}));

import { AgentPalette } from "./AgentPalette";

afterEach(() => {
  vi.clearAllMocks();
});

describe("AgentPalette (cross-feature: F04 registry → draggable nodes)", () => {
  it("renders each registry agent as a draggable item carrying its id and label", () => {
    useAgentsMock.mockReturnValue({ data: sampleAgents, isLoading: false, isError: false });
    render(<AgentPalette />);

    const items = screen.getAllByRole("listitem");
    expect(items).toHaveLength(2);

    const first = items[0];
    expect(first).toHaveAttribute("draggable", "true");
    expect(first).toHaveAttribute("data-agent-id", "agent-1");
    expect(within(first).getByText("Summarizer")).toBeInTheDocument();
    expect(within(first).getByText("gpt-4o")).toBeInTheDocument();
  });

  it("carries the agent id on the DataTransfer at drag start", () => {
    useAgentsMock.mockReturnValue({ data: sampleAgents, isLoading: false, isError: false });
    render(<AgentPalette />);

    const setData = vi.fn();
    const item = screen.getByText("Researcher").closest("li") as HTMLLIElement;
    item.dispatchEvent(
      Object.assign(new Event("dragstart", { bubbles: true }), {
        dataTransfer: { setData, effectAllowed: "" },
      }),
    );
    expect(setData).toHaveBeenCalledWith("application/x-agent-id", "agent-2");
  });

  it("shows an empty-state message when there are no agents", () => {
    useAgentsMock.mockReturnValue({ data: [], isLoading: false, isError: false });
    render(<AgentPalette />);
    expect(screen.getByText("No agents yet. Create one on the Agents dashboard.")).toBeInTheDocument();
  });

  it("offers a keyboard 'Add to canvas' fallback when onAddToCanvas is provided", async () => {
    useAgentsMock.mockReturnValue({ data: sampleAgents, isLoading: false, isError: false });
    const onAddToCanvas = vi.fn();
    render(<AgentPalette onAddToCanvas={onAddToCanvas} />);

    await userEvent.click(screen.getByRole("button", { name: "Add Summarizer to canvas" }));
    expect(onAddToCanvas).toHaveBeenCalledWith("agent-1");
  });

  it("omits the add button when no onAddToCanvas handler is given", () => {
    useAgentsMock.mockReturnValue({ data: sampleAgents, isLoading: false, isError: false });
    render(<AgentPalette />);
    expect(screen.queryByRole("button", { name: /Add .* to canvas/ })).not.toBeInTheDocument();
  });
});
