import type { NodeRunState, NodeStatus } from "../../lib/runStream";
import { StatusDot } from "../ui";
import styles from "./NodeBlock.module.css";

/** Human label per node status (canvas node mirrors this). */
const STATUS_LABEL: Record<NodeStatus, string> = {
  idle: "Idle",
  running: "Running",
  complete: "Complete",
  error: "Error",
  skipped: "Skipped",
};

const STATE_CLASS: Partial<Record<NodeStatus, string>> = {
  running: styles.running,
  complete: styles.complete,
  error: styles.error,
};

interface NodeBlockProps {
  node: NodeRunState;
}

/**
 * One agent block in the monitor panel (F10): the node's agent name + model, a
 * StatusDot reflecting its lifecycle, and its output — the final text once
 * complete, the streamed partial (with a caret) while running, or the error
 * message on failure.
 */
export function NodeBlock({ node }: NodeBlockProps) {
  const label = STATUS_LABEL[node.status];
  const isError = node.status === "error";
  const isRunning = node.status === "running";
  const body = isError ? node.error : (node.output ?? (node.partial.length > 0 ? node.partial : null));

  return (
    <article
      className={[styles.block, STATE_CLASS[node.status]].filter(Boolean).join(" ")}
      aria-label={`Agent block: ${node.agentName ?? node.nodeId}`}
    >
      <header className={styles.header}>
        <StatusDot status={node.status} label={`Status: ${label}`} />
        <span className={styles.name}>{node.agentName ?? node.nodeId}</span>
        {node.model && <span className={styles.model}>{node.model}</span>}
        {isRunning && (
          <span className={styles.streaming} aria-label="streaming">
            streaming…
          </span>
        )}
      </header>
      {body && (
        <p
          role={isError ? "alert" : undefined}
          className={[
            styles.body,
            isError && styles.bodyError,
            isRunning && !node.output && styles.caret,
          ]
            .filter(Boolean)
            .join(" ")}
        >
          {body}
        </p>
      )}
    </article>
  );
}
