import type { NodeRunState } from "../../lib/runStream";
import type { RunConnection } from "../../hooks/useRunStream";
import { Alert, Badge } from "../ui";
import { ConversationTurns, type ConversationTurn } from "./ConversationTurns";
import { NodeBlock } from "./NodeBlock";
import { PromptBar } from "./PromptBar";
import styles from "./ConversationMonitor.module.css";

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
 * The right-split conversational monitor (F10): a run-state chip, a
 * "Reconnecting…" indicator shown while an active run's stream is
 * re-establishing, the turn-based history, the live per-node agent blocks, and
 * the prompt bar pinned to the bottom. Rejection notices and the empty-output
 * message arrive as `system`/`assistant` turns.
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

  // The live nodes belong to the most recent run, so they slot between the
  // latest user prompt and the assistant response that follows it. Split the
  // turns at the last user message: history + prompt render above the nodes,
  // the response (and any post-prompt system notices) render below them.
  let lastUserIdx = -1;
  for (let i = turns.length - 1; i >= 0; i--) {
    if (turns[i].role === "user") {
      lastUserIdx = i;
      break;
    }
  }
  const headTurns = lastUserIdx >= 0 ? turns.slice(0, lastUserIdx + 1) : turns;
  const tailTurns = lastUserIdx >= 0 ? turns.slice(lastUserIdx + 1) : [];

  return (
    <section className={styles.monitor} aria-label="Conversation monitor">
      <div className={styles.header}>
        <h3 className={styles.title}>Monitor</h3>
        {isRunning ? (
          <Badge variant="running" dot>
            Running
          </Badge>
        ) : (
          <Badge variant="neutral" dot>
            Idle
          </Badge>
        )}
      </div>

      {reconnecting && (
        <Alert variant="warning" className={styles.reconnecting} title="Reconnecting…">
          Re-establishing the run stream.
        </Alert>
      )}

      <div className={styles.scroll}>
        {turns.length === 0 && nodes.length === 0 ? (
          // Empty state (no run yet): let ConversationTurns show its placeholder.
          <ConversationTurns turns={turns} />
        ) : (
          <>
            {headTurns.length > 0 && <ConversationTurns turns={headTurns} />}

            {nodes.length > 0 && (
              <div className={styles.nodes} aria-label="Agent blocks" aria-live="polite">
                <span className={styles.nodesLabel}>Live nodes</span>
                {nodes.map((node) => (
                  <NodeBlock key={node.nodeId} node={node} />
                ))}
              </div>
            )}

            {tailTurns.length > 0 && <ConversationTurns turns={tailTurns} />}
          </>
        )}
      </div>

      <div className={styles.prompt}>
        <PromptBar
          prompt={prompt}
          onPromptChange={onPromptChange}
          onSubmit={onSubmit}
          isRunning={isRunning}
          disabledReason={runDisabledReason}
        />
      </div>
    </section>
  );
}
