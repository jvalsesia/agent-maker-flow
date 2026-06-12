import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { NodeBlock } from "./NodeBlock";
import type { NodeRunState } from "../../lib/runStream";

function node(overrides: Partial<NodeRunState>): NodeRunState {
  return {
    nodeId: "n1",
    agentName: "Researcher",
    model: "gpt-4o",
    status: "idle",
    partial: "",
    output: null,
    error: null,
    ...overrides,
  };
}

describe("NodeBlock", () => {
  it("shows the running status light and streamed partial output", () => {
    render(<NodeBlock node={node({ status: "running", partial: "thinking…" })} />);
    expect(screen.getByLabelText("Status: Running")).toBeInTheDocument();
    expect(screen.getByText("thinking…")).toBeInTheDocument();
  });

  it("shows the complete status light and final output", () => {
    render(<NodeBlock node={node({ status: "complete", output: "final answer" })} />);
    expect(screen.getByLabelText("Status: Complete")).toBeInTheDocument();
    expect(screen.getByText("final answer")).toBeInTheDocument();
  });

  it("shows the error status light and the error message", () => {
    render(<NodeBlock node={node({ status: "error", error: "Model gateway unavailable." })} />);
    expect(screen.getByLabelText("Status: Error")).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("Model gateway unavailable.");
  });

  it("shows the skipped status light", () => {
    render(<NodeBlock node={node({ status: "skipped" })} />);
    expect(screen.getByLabelText("Status: Skipped")).toBeInTheDocument();
  });
});
