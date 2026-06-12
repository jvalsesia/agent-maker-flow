import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Input } from "./Input";
import { StatusDot } from "./StatusDot";
import { Badge } from "./Badge";
import { Alert } from "./Alert";

describe("Input", () => {
  it("associates its label with the control", () => {
    render(<Input label="Flow name" />);
    expect(screen.getByLabelText("Flow name")).toBeInstanceOf(HTMLInputElement);
  });

  it("exposes errors via role=alert and aria-invalid + aria-describedby", () => {
    render(<Input label="Name" error="Name is taken" />);
    const input = screen.getByLabelText("Name");
    expect(input).toHaveAttribute("aria-invalid", "true");
    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent("Name is taken");
    expect(input.getAttribute("aria-describedby")).toContain(alert.id);
  });

  it("renders a counter", () => {
    render(<Input label="Name" counter="3 / 80" />);
    expect(screen.getByText("3 / 80")).toBeInTheDocument();
  });
});

describe("StatusDot", () => {
  it("conveys status via an accessible label, not color alone", () => {
    render(<StatusDot status="running" />);
    expect(screen.getByRole("img", { name: "Running" })).toBeInTheDocument();
  });
});

describe("Badge", () => {
  it("renders its content", () => {
    render(<Badge variant="accent">★ Root</Badge>);
    expect(screen.getByText("★ Root")).toBeInTheDocument();
  });
});

describe("Alert", () => {
  it("uses role=alert for the danger variant", () => {
    render(<Alert variant="danger" title="Failed" />);
    expect(screen.getByRole("alert")).toHaveTextContent("Failed");
  });

  it("uses role=status for non-danger variants", () => {
    render(<Alert variant="warning">Heads up</Alert>);
    expect(screen.getByRole("status")).toHaveTextContent("Heads up");
  });
});
