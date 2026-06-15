import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ConversationMonitor } from "./ConversationMonitor";
import type { ConversationTurn } from "./ConversationTurns";
import { EMPTY_OUTPUT_MESSAGE, type NodeRunState } from "../../lib/runStream";
import type { RunConnection } from "../../hooks/useRunStream";

type Props = Parameters<typeof ConversationMonitor>[0];

function baseProps(overrides: Partial<Props> = {}): Props {
  return {
    turns: [],
    nodes: [],
    connection: "idle" as RunConnection,
    isRunning: false,
    prompt: "",
    onPromptChange: vi.fn(),
    onSubmit: vi.fn(),
    runDisabledReason: null,
    ...overrides,
  };
}

/** A stateful harness so "type then submit" exercises the controlled input. */
function Harness({ onSubmit }: { onSubmit: (p: string) => void }) {
  const [prompt, setPrompt] = useState("");
  return (
    <ConversationMonitor
      {...baseProps({ prompt, onPromptChange: setPrompt, onSubmit })}
    />
  );
}

const node = (overrides: Partial<NodeRunState>): NodeRunState => ({
  nodeId: "n1",
  agentName: "Researcher",
  model: "gpt-4o",
  status: "idle",
  partial: "",
  output: null,
  error: null,
  ...overrides,
});

describe("ConversationMonitor", () => {
  it("submitting a typed prompt invokes onSubmit with the trimmed text", async () => {
    const onSubmit = vi.fn();
    render(<Harness onSubmit={onSubmit} />);

    await userEvent.type(screen.getByLabelText("Prompt"), "  hello  ");
    await userEvent.click(screen.getByRole("button", { name: "Run Flow" }));

    expect(onSubmit).toHaveBeenCalledWith("hello");
  });

  it("blocks submit when the prompt is empty", async () => {
    const onSubmit = vi.fn();
    render(<Harness onSubmit={onSubmit} />);

    await userEvent.click(screen.getByRole("button", { name: "Run Flow" }));

    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("lights up node blocks by status", () => {
    render(
      <ConversationMonitor
        {...baseProps({
          nodes: [
            node({ nodeId: "n1", status: "complete", output: "done" }),
            node({ nodeId: "n2", agentName: "Writer", status: "running", partial: "draft" }),
          ],
        })}
      />,
    );

    expect(screen.getByLabelText("Status: Complete")).toBeInTheDocument();
    expect(screen.getByLabelText("Status: Running")).toBeInTheDocument();
    expect(screen.getByText("done")).toBeInTheDocument();
    expect(screen.getByText("draft")).toBeInTheDocument();
  });

  it("renders the aggregated final output as the assistant turn", () => {
    const turns: ConversationTurn[] = [
      { id: "user-1", role: "user", content: "Summarize" },
      { id: "assistant-1", role: "assistant", content: "Here is the summary." },
    ];
    render(<ConversationMonitor {...baseProps({ turns })} />);
    expect(screen.getByText("Here is the summary.")).toBeInTheDocument();
  });

  it("shows the completion notice when a run produced no output", () => {
    const turns: ConversationTurn[] = [
      { id: "assistant-1", role: "assistant", content: EMPTY_OUTPUT_MESSAGE },
    ];
    render(<ConversationMonitor {...baseProps({ turns })} />);
    expect(screen.getByText(EMPTY_OUTPUT_MESSAGE)).toBeInTheDocument();
  });

  it("renders a rejection as a system line", () => {
    const turns: ConversationTurn[] = [
      { id: "system-1", role: "system", content: "A run is already in progress for this flow." },
    ];
    render(<ConversationMonitor {...baseProps({ turns })} />);
    expect(screen.getByRole("alert")).toHaveTextContent(
      "A run is already in progress for this flow.",
    );
  });

  it("renders the assistant response after the live nodes", () => {
    const turns: ConversationTurn[] = [
      { id: "user-1", role: "user", content: "Summarize" },
      { id: "assistant-1", role: "assistant", content: "Here is the summary." },
    ];
    render(
      <ConversationMonitor
        {...baseProps({ turns, nodes: [node({ status: "complete", output: "done" })] })}
      />,
    );

    const nodesLabel = screen.getByText("Live nodes");
    const assistant = screen.getByText("Here is the summary.");
    const prompt = screen.getByText("Summarize");

    // Order in the scroll region: prompt -> live nodes -> assistant response.
    expect(prompt.compareDocumentPosition(nodesLabel)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    expect(nodesLabel.compareDocumentPosition(assistant)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
  });

  it("shows the reconnecting indicator while an active run is re-establishing", () => {
    render(<ConversationMonitor {...baseProps({ isRunning: true, connection: "connecting" })} />);
    expect(screen.getByText("Reconnecting…")).toBeInTheDocument();
  });
});
