import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { MissingAgentsBanner } from "./MissingAgentsBanner";

describe("MissingAgentsBanner", () => {
  it("renders nothing when no agents are missing", () => {
    const { container } = render(<MissingAgentsBanner missingAgentIds={[]} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("flags the count and lists each missing agent id", () => {
    render(<MissingAgentsBanner missingAgentIds={["agent-1", "agent-2"]} />);

    const banner = screen.getByRole("alert", { name: "Missing agents" });
    expect(banner).toHaveTextContent("2 node(s) reference an agent that no longer exists");
    expect(screen.getByText("agent-1")).toBeInTheDocument();
    expect(screen.getByText("agent-2")).toBeInTheDocument();
  });
});
