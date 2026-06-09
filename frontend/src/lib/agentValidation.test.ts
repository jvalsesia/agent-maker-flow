import { describe, expect, it } from "vitest";

import {
  NAME_MAX,
  PREAMBLE_MAX,
  SYSTEM_PROMPT_MAX,
  validateAgentForm,
  validateModel,
  validateName,
  validatePreamble,
  validateProvider,
  validateRecentN,
  validateSystemPrompt,
  validateTopK,
  type AgentFormValues,
} from "./agentValidation";

describe("validateName", () => {
  it("accepts a normal name", () => {
    expect(validateName("Summarizer")).toBeNull();
  });

  it("rejects empty and whitespace-only names", () => {
    expect(validateName("")).toBe("Name is required.");
    expect(validateName("   ")).toBe("Name is required.");
  });

  it("rejects names longer than the limit (trimmed)", () => {
    expect(validateName("a".repeat(NAME_MAX))).toBeNull();
    expect(validateName("a".repeat(NAME_MAX + 1))).toBe(
      `Name must be ${NAME_MAX} characters or fewer.`,
    );
  });
});

describe("validatePreamble", () => {
  it("accepts an empty or within-limit preamble", () => {
    expect(validatePreamble("")).toBeNull();
    expect(validatePreamble("p".repeat(PREAMBLE_MAX))).toBeNull();
  });

  it("rejects an over-limit preamble", () => {
    expect(validatePreamble("p".repeat(PREAMBLE_MAX + 1))).toBe(
      `Preamble must be ${PREAMBLE_MAX} characters or fewer.`,
    );
  });
});

describe("validateSystemPrompt", () => {
  it("requires a non-empty prompt", () => {
    expect(validateSystemPrompt("   ")).toBe("System prompt is required.");
  });

  it("rejects an over-limit prompt", () => {
    expect(validateSystemPrompt("s".repeat(SYSTEM_PROMPT_MAX + 1))).toBe(
      `System prompt must be ${SYSTEM_PROMPT_MAX} characters or fewer.`,
    );
  });

  it("accepts a valid prompt", () => {
    expect(validateSystemPrompt("Summarize the input.")).toBeNull();
  });
});

describe("range validators", () => {
  it("bounds recent_n to 0–100", () => {
    expect(validateRecentN(0)).toBeNull();
    expect(validateRecentN(100)).toBeNull();
    expect(validateRecentN(-1)).toBe("Value must be between 0 and 100.");
    expect(validateRecentN(200)).toBe("Value must be between 0 and 100.");
    expect(validateRecentN(1.5)).toBe("Value must be between 0 and 100.");
  });

  it("bounds top_k to 0–50", () => {
    expect(validateTopK(0)).toBeNull();
    expect(validateTopK(50)).toBeNull();
    expect(validateTopK(99)).toBe("Value must be between 0 and 50.");
  });
});

describe("provider/model presence", () => {
  it("requires a provider and model", () => {
    expect(validateProvider("")).toBe("Select a provider.");
    expect(validateProvider("openai")).toBeNull();
    expect(validateModel("")).toBe("Select a model.");
    expect(validateModel("gpt-4o")).toBeNull();
  });
});

describe("validateAgentForm", () => {
  const valid: AgentFormValues = {
    name: "Summarizer",
    preamble: "",
    system_prompt: "Summarize the input.",
    provider: "openai",
    model: "gpt-4o",
    recent_n: 10,
    top_k: 5,
  };

  it("returns no errors for a valid form", () => {
    expect(validateAgentForm(valid)).toEqual({});
  });

  it("collects only the failing fields", () => {
    const errors = validateAgentForm({
      ...valid,
      name: "",
      recent_n: 200,
      model: "",
    });
    expect(errors).toEqual({
      name: "Name is required.",
      recent_n: "Value must be between 0 and 100.",
      model: "Select a model.",
    });
  });
});
