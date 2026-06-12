import type { NodeRunState, NodeStatus } from "../../lib/runStream";

/** Human label + light colour per node status (canvas badge mirrors this). */
const STATUS_STYLE: Record<NodeStatus, { label: string; color: string }> = {
  idle: { label: "Idle", color: "#9aa0a6" },
  running: { label: "Running", color: "#1a73e8" },
  complete: { label: "Complete", color: "#188038" },
  error: { label: "Error", color: "#b00020" },
  skipped: { label: "Skipped", color: "#9aa0a6" },
};

interface NodeBlockProps {
  node: NodeRunState;
}

/**
 * One agent block in the monitor panel (F10): the node's agent name + model, a
 * status light reflecting its lifecycle, and its output — the final text once
 * complete, the streamed partial while running, or the error message on failure.
 */
export function NodeBlock({ node }: NodeBlockProps) {
  const style = STATUS_STYLE[node.status];
  const body =
    node.status === "error"
      ? node.error
      : node.output ?? (node.partial.length > 0 ? node.partial : null);

  return (
    <article
      aria-label={`Agent block: ${node.agentName ?? node.nodeId}`}
      style={{ border: "1px solid #ddd", borderRadius: 6, padding: 8, marginBottom: 8 }}
    >
      <header style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <span
          aria-label={`Status: ${style.label}`}
          title={style.label}
          style={{
            width: 10,
            height: 10,
            borderRadius: "50%",
            background: style.color,
            display: "inline-block",
          }}
        />
        <strong>{node.agentName ?? node.nodeId}</strong>
        {node.model && <small style={{ color: "#666" }}>{node.model}</small>}
        {node.status === "running" && <small aria-label="streaming">streaming…</small>}
      </header>
      {body && (
        <p
          role={node.status === "error" ? "alert" : undefined}
          style={{
            margin: "6px 0 0",
            whiteSpace: "pre-wrap",
            color: node.status === "error" ? "#b00020" : undefined,
          }}
        >
          {body}
        </p>
      )}
    </article>
  );
}
