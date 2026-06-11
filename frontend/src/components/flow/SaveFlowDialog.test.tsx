import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { SaveFlowDialog } from "./SaveFlowDialog";

const noop = () => {};

describe("SaveFlowDialog name validation", () => {
  it("requires a name and disables submit until one is entered", () => {
    render(<SaveFlowDialog mode="save" onSubmit={noop} onCancel={noop} />);

    expect(screen.getByRole("dialog", { name: "Save flow" })).toBeInTheDocument();
    expect(screen.getByText("Name is required.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });

  it("rejects a name longer than 80 characters", () => {
    render(<SaveFlowDialog mode="save" onSubmit={noop} onCancel={noop} />);

    fireEvent.change(screen.getByLabelText("Name"), { target: { value: "a".repeat(81) } });

    expect(screen.getByText("Name must be 80 characters or fewer.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });

  it("submits the trimmed name once valid", async () => {
    const onSubmit = vi.fn();
    render(<SaveFlowDialog mode="save" onSubmit={onSubmit} onCancel={noop} />);

    await userEvent.type(screen.getByLabelText("Name"), "  Research pipeline  ");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(onSubmit).toHaveBeenCalledWith("Research pipeline");
  });

  it("shows the server's duplicate-name message", () => {
    render(
      <SaveFlowDialog
        mode="save"
        initialName="Pipeline"
        onSubmit={noop}
        onCancel={noop}
        error="A flow named 'Pipeline' already exists."
      />,
    );
    expect(
      screen.getByText("A flow named 'Pipeline' already exists."),
    ).toBeInTheDocument();
  });
});

describe("SaveFlowDialog rename mode", () => {
  it("prefills the current name and labels the action 'Rename'", async () => {
    const onSubmit = vi.fn();
    render(
      <SaveFlowDialog
        mode="rename"
        initialName="Old name"
        onSubmit={onSubmit}
        onCancel={noop}
      />,
    );

    expect(screen.getByRole("dialog", { name: "Rename flow" })).toBeInTheDocument();
    const input = screen.getByLabelText("Name") as HTMLInputElement;
    expect(input.value).toBe("Old name");

    await userEvent.clear(input);
    await userEvent.type(input, "New name");
    await userEvent.click(screen.getByRole("button", { name: "Rename" }));

    expect(onSubmit).toHaveBeenCalledWith("New name");
  });
});
