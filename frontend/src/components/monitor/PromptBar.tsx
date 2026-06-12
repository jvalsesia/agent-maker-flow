import type { FormEvent, KeyboardEvent } from "react";

import { Button } from "../ui";
import styles from "./PromptBar.module.css";

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
 * The prompt input bar (F10): a mono textarea plus a "Run Flow" submit.
 * Submission is blocked while a run is in flight, when the prompt is
 * empty/whitespace, or when the graph is not runnable (`disabledReason`).
 * Cmd/Ctrl+Enter submits. The trimmed prompt is passed to `onSubmit`.
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

  const submit = () => {
    if (blocked) return;
    onSubmit(trimmed);
  };

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault();
    submit();
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      submit();
    }
  };

  return (
    <form className={styles.bar} onSubmit={handleSubmit} aria-label="Run prompt">
      <textarea
        className={styles.textarea}
        aria-label="Prompt"
        value={prompt}
        onChange={(e) => onPromptChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Enter a message to run the flow…  (⌘/Ctrl+Enter)"
        rows={3}
        disabled={isRunning}
      />
      <div className={styles.footer}>
        <Button
          type="submit"
          variant="primary"
          disabled={blocked}
          loading={isRunning}
          loadingLabel="Running…"
        >
          Run Flow
        </Button>
        {disabledReason && !isRunning && (
          <span className={styles.reason} role="status">
            {disabledReason}
          </span>
        )}
      </div>
    </form>
  );
}
