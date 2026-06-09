import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

// The form pulls the provider/model catalog from the F03 hooks; mock them with
// a provider-aware model list so the dependency behavior is deterministic.
vi.mock("../../lib/models", () => ({
  useProviders: () => ({
    data: [
      { id: "openai", model_count: 1 },
      { id: "anthropic", model_count: 1 },
    ],
    isLoading: false,
    isSuccess: true,
  }),
  useModels: (provider: string | null) => ({
    data:
      provider === "openai"
        ? [{ id: "gpt-4o", label: "gpt-4o", mode: "chat" }]
        : provider === "anthropic"
          ? [{ id: "claude-3-5-sonnet", label: "claude-3-5-sonnet", mode: "chat" }]
          : undefined,
    isLoading: false,
    isSuccess: provider != null,
  }),
}));

import { ApiClientError } from "../../lib/apiClient";
import type { Agent } from "../../lib/agents";
import { AgentForm } from "./AgentForm";

afterEach(() => {
  vi.clearAllMocks();
});

const sampleAgent: Agent = {
  id: "agent-1",
  name: "Original",
  preamble: null,
  system_prompt: "Do the thing.",
  provider: "openai",
  model: "gpt-4o",
  recent_n: 10,
  top_k: 5,
  created_at: "2026-06-09T00:00:00Z",
  updated_at: "2026-06-09T00:00:00Z",
};

function renderForm(overrides: Partial<Parameters<typeof AgentForm>[0]> = {}) {
  const onSubmit = overrides.onSubmit ?? vi.fn().mockResolvedValue(undefined);
  const onCancel = overrides.onCancel ?? vi.fn();
  render(
    <AgentForm mode={overrides.mode ?? "create"} initial={overrides.initial} onSubmit={onSubmit} onCancel={onCancel} />,
  );
  return { onSubmit, onCancel };
}

describe("AgentForm provider/model dependency", () => {
  it("disables the model dropdown until a provider is chosen", () => {
    renderForm();
    expect(screen.getByLabelText("Model")).toBeDisabled();
  });

  it("enables and populates the model dropdown once a provider is selected", async () => {
    const user = userEvent.setup();
    renderForm();

    await user.selectOptions(screen.getByLabelText("Provider"), "openai");

    expect(screen.getByLabelText("Model")).toBeEnabled();
    expect(screen.getByRole("option", { name: "gpt-4o" })).toBeInTheDocument();
  });

  it("clears an incompatible model when the provider changes", async () => {
    const user = userEvent.setup();
    renderForm();

    await user.selectOptions(screen.getByLabelText("Provider"), "openai");
    await user.selectOptions(screen.getByLabelText("Model"), "gpt-4o");
    expect((screen.getByLabelText("Model") as HTMLSelectElement).value).toBe("gpt-4o");

    await user.selectOptions(screen.getByLabelText("Provider"), "anthropic");

    expect((screen.getByLabelText("Model") as HTMLSelectElement).value).toBe("");
    expect(screen.queryByRole("option", { name: "gpt-4o" })).not.toBeInTheDocument();
    expect(screen.getByRole("option", { name: "claude-3-5-sonnet" })).toBeInTheDocument();
  });
});

describe("AgentForm inline validation", () => {
  it("shows inline messages for an empty name and an out-of-range integer", async () => {
    const user = userEvent.setup();
    renderForm();

    const recentN = screen.getByLabelText("Recent messages (recent-N)");
    await user.clear(recentN);
    await user.type(recentN, "200");
    expect(screen.getByText("Value must be between 0 and 100.")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Create agent" }));
    expect(screen.getByText("Name is required.")).toBeInTheDocument();
  });

  it("maps an AGENT_NAME_TAKEN error to the name field", async () => {
    const user = userEvent.setup();
    const onSubmit = vi
      .fn()
      .mockRejectedValue(new ApiClientError({ code: "AGENT_NAME_TAKEN", message: "taken" }));
    renderForm({ onSubmit });

    await user.type(screen.getByLabelText("Name"), "Dup");
    await user.type(screen.getByLabelText("System prompt"), "Summarize.");
    await user.selectOptions(screen.getByLabelText("Provider"), "openai");
    await user.selectOptions(screen.getByLabelText("Model"), "gpt-4o");
    await user.click(screen.getByRole("button", { name: "Create agent" }));

    expect(onSubmit).toHaveBeenCalledTimes(1);
    expect(await screen.findByText("An agent named 'Dup' already exists.")).toBeInTheDocument();
  });
});

describe("AgentForm duplicate mode", () => {
  it("prefills the name with a (copy) suffix", () => {
    renderForm({ mode: "duplicate", initial: sampleAgent });
    expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe("Original (copy)");
  });
});
