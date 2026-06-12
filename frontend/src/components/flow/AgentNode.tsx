import { createContext, useContext } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";

import type { Agent } from "../../lib/agents";
import type { FlowNode } from "../../lib/flowGraph";
import type { NodeStatus } from "../../lib/runStream";
import { Badge, StatusDot } from "../ui";
import styles from "./AgentNode.module.css";

/** Human label per live run status (mirrors the monitor's NodeBlock). */
const STATUS_LABEL: Record<NodeStatus, string> = {
  idle: "Idle",
  running: "Running",
  complete: "Complete",
  error: "Error",
  skipped: "Skipped",
};

/** Maps run status onto the state class that recolors the node border. */
const STATE_CLASS: Partial<Record<NodeStatus, string>> = {
  running: styles.running,
  complete: styles.complete,
  error: styles.error,
};

/**
 * Per-canvas context consumed by every {@link AgentNode}. Keeping the agent
 * lookup, the root assignment, and the node-operation callbacks here lets the
 * node's `data` stay the serializable `{ agentId }` contract (F08/F09) instead
 * of carrying functions or denormalized agent fields.
 */
export interface FlowNodeContextValue {
  /** Registry agents keyed by id, for label + missing-agent resolution. */
  agentsById: Map<string, Agent>;
  /** The single Root Agent (null until assigned). */
  rootNodeId: string | null;
  /** Live per-node run status (F10); empty/absent when no run is active. */
  nodeStatuses?: Record<string, NodeStatus>;
  onSetRoot: (nodeId: string) => void;
  onDuplicate: (nodeId: string) => void;
  onDetach: (nodeId: string) => void;
  onDelete: (nodeId: string) => void;
}

export const FlowNodeContext = createContext<FlowNodeContextValue | null>(null);

/**
 * Custom React Flow node (F07): shows the referenced agent's name and model,
 * input/output handles, a Root badge when it is the root, an "Agent missing"
 * flag when the referenced agent was deleted from the registry, and the
 * per-node actions (set root, duplicate, detach, delete). Border + StatusDot
 * reflect the live run status within ~1s of the SSE event (F10, spec §5.2).
 */
export function AgentNode({ id, data }: NodeProps<FlowNode>) {
  const ctx = useContext(FlowNodeContext);
  const agent = ctx?.agentsById.get(data.agentId);
  const isRoot = ctx?.rootNodeId === id;
  const isMissing = !agent;
  const status = ctx?.nodeStatuses?.[id];

  const stateClass = isMissing ? styles.missing : status ? STATE_CLASS[status] : undefined;

  return (
    <div
      className={[styles.node, stateClass].filter(Boolean).join(" ")}
      aria-label={agent ? `Agent node: ${agent.name}` : "Agent node: missing agent"}
    >
      <Handle
        type="target"
        position={Position.Left}
        className={styles.handle}
        aria-label="Input connection"
      />

      <div className={styles.header}>
        {status && <StatusDot status={status} label={`Status: ${STATUS_LABEL[status]}`} />}
        {isMissing ? (
          <span className={styles.name} role="alert">
            <span className={styles.missingText}>Agent missing</span>
          </span>
        ) : (
          <span className={styles.name}>{agent.name}</span>
        )}
        {isRoot && (
          <span aria-label="Root Agent">
            <Badge variant="accent">★ Root</Badge>
          </span>
        )}
      </div>

      {!isMissing && (
        <div className={styles.model}>
          <StatusDot status="idle" label="" />
          {agent.model}
        </div>
      )}

      {status && (
        <div className={styles.statusRow}>
          <span>status: {STATUS_LABEL[status]}</span>
        </div>
      )}
      {status === "running" && <div className={styles.runningBar} aria-hidden="true" />}

      <div className={styles.actions}>
        <button
          type="button"
          className={[styles.action, styles.setRoot, isRoot && styles.setRootActive]
            .filter(Boolean)
            .join(" ")}
          aria-pressed={isRoot}
          aria-label={isRoot ? "Root agent" : "Set root"}
          title={isRoot ? "Root agent" : "Set as root agent"}
          onClick={() => ctx?.onSetRoot(id)}
        >
          {isRoot ? "★ Root" : "Set root"}
        </button>
        <button
          type="button"
          className={styles.action}
          aria-label="Duplicate"
          title="Duplicate node"
          onClick={() => ctx?.onDuplicate(id)}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
            <rect x="9" y="9" width="11" height="11" rx="2" />
            <path d="M5 15V5a2 2 0 012-2h10" />
          </svg>
        </button>
        <button
          type="button"
          className={styles.action}
          aria-label="Detach"
          title="Detach connections"
          onClick={() => ctx?.onDetach(id)}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
            <path strokeLinecap="round" d="M9 15l-3 3a3 3 0 01-4-4l3-3m7-1l3-3a3 3 0 014 4l-3 3M8 16L16 8" />
          </svg>
        </button>
        <button
          type="button"
          className={styles.action}
          aria-label="Delete"
          title="Delete node"
          onClick={() => ctx?.onDelete(id)}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
            <path strokeLinecap="round" d="M4 7h16M9 7V5a1 1 0 011-1h4a1 1 0 011 1v2m-8 0l1 13h8l1-13" />
          </svg>
        </button>
      </div>

      <Handle
        type="source"
        position={Position.Right}
        className={styles.handle}
        aria-label="Output connection"
      />
    </div>
  );
}
