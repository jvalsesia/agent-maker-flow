import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";

import { EmbeddingModelSelect } from "./EmbeddingModelSelect";
import type { Model } from "../lib/models";

const models: Model[] = [
  { id: "text-embedding-3-small", label: "text-embedding-3-small", mode: "embedding" },
  { id: "text-embedding-3-large", label: "text-embedding-3-large", mode: "embedding" },
  { id: "gpt-4o", label: "gpt-4o", mode: "chat" },
];

describe("EmbeddingModelSelect", () => {
  it("lists only embedding-mode models (cross-feature: F03 catalog → F05 selector)", () => {
    render(<EmbeddingModelSelect models={models} value="" onChange={vi.fn()} />);

    expect(screen.getByRole("option", { name: "text-embedding-3-small" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "text-embedding-3-large" })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: "gpt-4o" })).not.toBeInTheDocument();
  });

  it("warns when existing records use a different model", () => {
    render(
      <EmbeddingModelSelect
        models={models}
        value="text-embedding-3-small"
        onChange={vi.fn()}
        modelsInUse={["text-embedding-3-large"]}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("text-embedding-3-large");
  });

  it("does not warn when all records match the selected model", () => {
    render(
      <EmbeddingModelSelect
        models={models}
        value="text-embedding-3-small"
        onChange={vi.fn()}
        modelsInUse={["text-embedding-3-small"]}
      />,
    );
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
