import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { Button } from "./Button";
import { Modal } from "./Modal";

function Harness({ busy = false }: { busy?: boolean }) {
  return (
    <Modal open onClose={() => {}} title="Confirm" busy={busy} footer={<Button>OK</Button>}>
      <p>Body text</p>
    </Modal>
  );
}

describe("Modal", () => {
  it("renders as an accessible dialog labelled by its title", () => {
    render(<Harness />);
    const dialog = screen.getByRole("dialog");
    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(dialog).toHaveAccessibleName("Confirm");
  });

  it("closes on Escape", async () => {
    const onClose = vi.fn();
    render(
      <Modal open onClose={onClose} title="Confirm">
        <p>Body</p>
      </Modal>,
    );
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("closes when the scrim is clicked", async () => {
    const onClose = vi.fn();
    render(
      <Modal open onClose={onClose} title="Confirm">
        <p>Body</p>
      </Modal>,
    );
    // The scrim is the dialog's offset parent overlay; click outside the panel.
    const dialog = screen.getByRole("dialog");
    const scrim = dialog.parentElement as HTMLElement;
    await userEvent.pointer({ keys: "[MouseLeft]", target: scrim });
    expect(onClose).toHaveBeenCalled();
  });

  it("does not close on Escape while busy", async () => {
    const onClose = vi.fn();
    render(
      <Modal open onClose={onClose} title="Confirm" busy>
        <p>Body</p>
      </Modal>,
    );
    await userEvent.keyboard("{Escape}");
    expect(onClose).not.toHaveBeenCalled();
  });

  it("renders nothing when closed", () => {
    render(
      <Modal open={false} onClose={() => {}} title="Confirm">
        <p>Body</p>
      </Modal>,
    );
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
