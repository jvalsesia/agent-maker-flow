import type { NodeRunState } from "../../lib/runStream";
import type { RunConnection } from "../../hooks/useRunStream";
import { ConversationTurns, type ConversationTurn } from "./ConversationTurns";
import { NodeBlock } from "./NodeBlock";
import { PromptBar } from "./PromptBar";

interface ConversationMonitorProps {
  turns: ConversationTurn[];
  /** The run's nodes in execution-discovery order (from `RunState.nodes`). */
  nodes: NodeRunState[];
  connection: RunConnection;
  isRunning: boolean;
  prompt: string;
  onPromptChange: (value: string) => void;
  onSubmit: (prompt: string) => void;
  /** Non-null disables the run control and explains why. */
  runDisabledReason?: string | null;
}

/**
 * The right-split conversational monitor (F10): the prompt bar, the turn-based
 * history, the live per-node agent blocks, and a "Reconnecting…" indicator
 * shown while an active run's stream is re-establishing. Rejection notices and
 * the empty-output message arrive as `system`/`assistant` turns.
 */
export function ConversationMonitor({
  turns,
  nodes,
  connection,
  isRunning,
  prompt,
  onPromptChange,
  onSubmit,
  runDisabledReason,
}: ConversationMonitorProps) {
  const reconnecting = isRunning && connection === "connecting";

  return (
    <section
      aria-label="Conversation monitor"
      style={{ display: "flex", flexDirection: "column", gap: 12, height: "100%" }}
    >
      <h3 style={{ margin: 0 }}>Monitor</h3>

      {reconnecting && <p role="status">Reconnecting…</p>}

      <ConversationTurns turns={turns} />

      {nodes.length > 0 && (
        <div aria-label="Agent blocks">
          {nodes.map((node) => (
            <NodeBlock key={node.nodeId} node={node} />
          ))}
        </div>
      )}

      <PromptBar
        prompt={prompt}
        onPromptChange={onPromptChange}
        onSubmit={onSubmit}
        isRunning={isRunning}
        disabledReason={runDisabledReason}
      />
    </section>
  );
}
