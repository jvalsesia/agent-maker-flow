import { type ComponentProps } from "react";
import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ReactFlowProvider } from "@xyflow/react";

import type { Agent } from "../../lib/agents";
import { AgentNode, FlowNodeContext, type FlowNodeContextValue } from "./AgentNode";

const agent: Agent = {
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
};

/** Build the React Flow NodeProps the component reads (id + data). */
function nodeProps(nodeId: string, agentId: string): ComponentProps<typeof AgentNode> {
  return { id: nodeId, data: { agentId } } as unknown as ComponentProps<typeof AgentNode>;
}

function renderNode(
  props: ComponentProps<typeof AgentNode>,
  ctx: Partial<FlowNodeContextValue> = {},
) {
  const value: FlowNodeContextValue = {
    agentsById: new Map([[agent.id, agent]]),
    rootNodeId: null,
    onSetRoot: vi.fn(),
    onDuplicate: vi.fn(),
    onDetach: vi.fn(),
    onDelete: vi.fn(),
    ...ctx,
  };
  render(
    <ReactFlowProvider>
      <FlowNodeContext.Provider value={value}>
        <AgentNode {...props} />
      </FlowNodeContext.Provider>
    </ReactFlowProvider>,
  );
  return value;
}

describe("AgentNode", () => {
  it("renders the referenced agent's name and model", () => {
    renderNode(nodeProps("node-1", "agent-1"));
    expect(screen.getByText("Summarizer")).toBeInTheDocument();
    expect(screen.getByText("gpt-4o")).toBeInTheDocument();
  });

  it("shows the Root badge only when the node is the root", () => {
    renderNode(nodeProps("node-1", "agent-1"), { rootNodeId: "node-1" });
    expect(screen.getByLabelText("Root Agent")).toBeInTheDocument();
  });

  it("does not show the Root badge for a non-root node", () => {
    renderNode(nodeProps("node-1", "agent-1"), { rootNodeId: "node-2" });
    expect(screen.queryByLabelText("Root Agent")).not.toBeInTheDocument();
  });

  it("flags a node whose referenced agent is missing from the registry", () => {
    renderNode(nodeProps("node-1", "agent-gone"), { agentsById: new Map() });
    expect(screen.getByText("Agent missing")).toBeInTheDocument();
  });

  it("invokes the set-root callback for its own node id", async () => {
    const value = renderNode(nodeProps("node-7", "agent-1"));
    await userEvent.click(screen.getByRole("button", { name: "Set root" }));
    expect(value.onSetRoot).toHaveBeenCalledWith("node-7");
  });
});
