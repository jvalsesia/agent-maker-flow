import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { SavedFlowsList } from "./SavedFlowsList";
import type { FlowSummary } from "../../lib/flows";

const flows: FlowSummary[] = [
  {
    id: "flow-1",
    name: "Research pipeline",
    created_at: "2026-06-10T14:00:00Z",
    updated_at: "2026-06-10T14:05:00Z",
  },
  {
    id: "flow-2",
    name: "Triage bot",
    created_at: "2026-06-09T10:00:00Z",
    updated_at: "2026-06-09T10:30:00Z",
  },
];

const noop = () => {};

describe("SavedFlowsList", () => {
  it("shows an empty state when there are no flows", () => {
    render(
      <SavedFlowsList flows={[]} onOpen={noop} onRename={noop} onDelete={noop} />,
    );
    expect(screen.getByText(/No saved flows yet/)).toBeInTheDocument();
  });

  it("renders a row per flow with a last-updated time", () => {
    render(
      <SavedFlowsList flows={flows} onOpen={noop} onRename={noop} onDelete={noop} />,
    );
    expect(screen.getByText("Research pipeline")).toBeInTheDocument();
    expect(screen.getByText("Triage bot")).toBeInTheDocument();
    // One <time> per row carries the raw timestamp for the last-updated indicator.
    const times = screen.getAllByText((_, el) => el?.tagName.toLowerCase() === "time");
    expect(times).toHaveLength(2);
  });

  it("fires open, rename, and delete with the row's flow", async () => {
    const onOpen = vi.fn();
    const onRename = vi.fn();
    const onDelete = vi.fn();
    render(
      <SavedFlowsList
        flows={flows}
        onOpen={onOpen}
        onRename={onRename}
        onDelete={onDelete}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "Open Research pipeline" }));
    await userEvent.click(screen.getByRole("button", { name: "Rename Triage bot" }));
    await userEvent.click(screen.getByRole("button", { name: "Delete Research pipeline" }));

    expect(onOpen).toHaveBeenCalledWith(flows[0]);
    expect(onRename).toHaveBeenCalledWith(flows[1]);
    expect(onDelete).toHaveBeenCalledWith(flows[0]);
  });

  it("marks the active flow's row as current", () => {
    render(
      <SavedFlowsList
        flows={flows}
        activeFlowId="flow-2"
        onOpen={noop}
        onRename={noop}
        onDelete={noop}
      />,
    );
    const activeRow = screen.getByText("Triage bot").closest("tr");
    expect(activeRow).toHaveAttribute("aria-current", "true");
  });
});
