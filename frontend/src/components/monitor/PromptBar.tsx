import type { FormEvent } from "react";

interface PromptBarProps {
  prompt: string;
  onPromptChange: (value: string) => void;
  /** Called with the trimmed prompt when the user submits a runnable flow. */
  onSubmit: (prompt: string) => void;
  /** True while a run is in flight; submission is blocked. */
  isRunning: boolean;
  /** Non-null disables submission and explains why (e.g. assign a root agent). */
  disabledReason?: string | null;
}

/**
 * The prompt input bar (F10): a textarea plus a "Run Flow" submit. Submission
 * is blocked while a run is in flight, when the prompt is empty/whitespace, or
 * when the graph is not runnable (`disabledReason`). The trimmed prompt is
 * passed to `onSubmit`.
 */
export function PromptBar({
  prompt,
  onPromptChange,
  onSubmit,
  isRunning,
  disabledReason,
}: PromptBarProps) {
  const trimmed = prompt.trim();
  const blocked = isRunning || trimmed.length === 0 || Boolean(disabledReason);

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault();
    if (blocked) return;
    onSubmit(trimmed);
  };

  return (
    <form onSubmit={handleSubmit} aria-label="Run prompt">
      <textarea
        aria-label="Prompt"
        value={prompt}
        onChange={(e) => onPromptChange(e.target.value)}
        placeholder="Enter a message to run the flow…"
        rows={3}
        disabled={isRunning}
        style={{ width: "100%", resize: "vertical", boxSizing: "border-box" }}
      />
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 6 }}>
        <button type="submit" disabled={blocked}>
          {isRunning ? "Running…" : "Run Flow"}
        </button>
        {disabledReason && !isRunning && <span role="status">{disabledReason}</span>}
      </div>
    </form>
  );
}
