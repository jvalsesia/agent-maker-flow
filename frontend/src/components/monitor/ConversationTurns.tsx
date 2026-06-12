import { EmptyState } from "../ui";
import styles from "./ConversationTurns.module.css";

/** One entry in the session conversation: the prompt, the aggregated answer, or a notice. */
export interface ConversationTurn {
  /** Stable key (derived from the run id + role). */
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
}

const ROLE_LABEL: Record<ConversationTurn["role"], string> = {
  user: "You",
  assistant: "Assistant",
  system: "System",
};

const TURN_CLASS: Record<ConversationTurn["role"], string> = {
  user: styles.user,
  assistant: styles.assistant,
  system: styles.system,
};

const BUBBLE_CLASS: Record<ConversationTurn["role"], string> = {
  user: styles.userBubble,
  assistant: styles.assistantBubble,
  system: styles.systemBubble,
};

interface ConversationTurnsProps {
  turns: ConversationTurn[];
}

/**
 * Render the accumulated conversation turns as chat bubbles (F10): user right,
 * assistant left, system full-width. `system` turns carry rejection notices and
 * the empty-output message; they use `role="alert"` so the notice surfaces
 * immediately.
 */
export function ConversationTurns({ turns }: ConversationTurnsProps) {
  if (turns.length === 0) {
    return (
      <EmptyState
        icon={
          <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
            <path strokeLinecap="round" strokeLinejoin="round" d="M21 12a8 8 0 01-11.5 7.2L3 21l1.8-6.5A8 8 0 1121 12z" />
          </svg>
        }
        title="No messages yet"
        description="Enter a prompt and run the flow to see the conversation here."
      />
    );
  }

  return (
    <ol className={styles.list} aria-label="Conversation history">
      {turns.map((turn) => (
        <li key={turn.id} className={[styles.turn, TURN_CLASS[turn.role]].join(" ")}>
          <span className={styles.role}>{ROLE_LABEL[turn.role]}</span>
          <span
            role={turn.role === "system" ? "alert" : undefined}
            className={[styles.bubble, BUBBLE_CLASS[turn.role]].join(" ")}
          >
            {turn.content}
          </span>
        </li>
      ))}
    </ol>
  );
}
