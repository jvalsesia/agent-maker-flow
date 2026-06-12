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

interface ConversationTurnsProps {
  turns: ConversationTurn[];
}

/**
 * Render the accumulated conversation turns in order (F10). `system` turns
 * carry rejection notices and the empty-output message; they use `role="alert"`
 * so the rejection surfaces immediately.
 */
export function ConversationTurns({ turns }: ConversationTurnsProps) {
  if (turns.length === 0) {
    return (
      <p style={{ color: "#666" }}>No messages yet — enter a prompt and run the flow.</p>
    );
  }

  return (
    <ol aria-label="Conversation history" style={{ listStyle: "none", padding: 0, margin: 0 }}>
      {turns.map((turn) => (
        <li key={turn.id} style={{ marginBottom: 8 }}>
          <strong>{ROLE_LABEL[turn.role]}:</strong>{" "}
          <span role={turn.role === "system" ? "alert" : undefined}>{turn.content}</span>
        </li>
      ))}
    </ol>
  );
}
