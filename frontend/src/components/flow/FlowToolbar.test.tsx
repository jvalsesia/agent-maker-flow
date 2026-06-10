import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { FlowToolbar } from "./FlowToolbar";

describe("FlowToolbar Run-Flow disabled rules", () => {
  it("disables Run and prompts to add a node when the canvas is empty", () => {
    render(<FlowToolbar hasNodes={false} hasRoot={false} missingAgentCount={0} />);
    expect(screen.getByRole("button", { name: "Run Flow" })).toBeDisabled();
    expect(screen.getByText("Add an agent to the canvas before running.")).toBeInTheDocument();
  });

  it("disables Run and prompts to assign a root when nodes exist but no root", () => {
    render(<FlowToolbar hasNodes hasRoot={false} missingAgentCount={0} />);
    expect(screen.getByRole("button", { name: "Run Flow" })).toBeDisabled();
    expect(screen.getByText("Assign a Root Agent before running.")).toBeInTheDocument();
  });

  it("disables Run when a referenced agent is missing", () => {
    render(<FlowToolbar hasNodes hasRoot missingAgentCount={1} />);
    expect(screen.getByRole("button", { name: "Run Flow" })).toBeDisabled();
    expect(screen.getByText("Resolve missing agents before running.")).toBeInTheDocument();
  });

  it("enables Run once a root is assigned, a node exists, and no agent is missing", async () => {
    const onRun = vi.fn();
    render(<FlowToolbar hasNodes hasRoot missingAgentCount={0} onRun={onRun} />);
    const button = screen.getByRole("button", { name: "Run Flow" });
    expect(button).toBeEnabled();
    await userEvent.click(button);
    expect(onRun).toHaveBeenCalledTimes(1);
  });
});
