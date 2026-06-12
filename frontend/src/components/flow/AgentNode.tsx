import { createContext, useContext } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";

import type { Agent } from "../../lib/agents";
import type { FlowNode } from "../../lib/flowGraph";
import type { NodeStatus } from "../../lib/runStream";

/** Badge label + colour per live run status (mirrors the monitor's NodeBlock). */
const STATUS_BADGE: Record<NodeStatus, { label: string; color: string }> = {
  idle: { label: "Idle", color: "#9aa0a6" },
  running: { label: "Running", color: "#1a73e8" },
  complete: { label: "Complete", color: "#188038" },
  error: { label: "Error", color: "#b00020" },
  skipped: { label: "Skipped", color: "#9aa0a6" },
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
 * per-node actions (set root, duplicate, detach, delete).
 */
export function AgentNode({ id, data }: NodeProps<FlowNode>) {
  const ctx = useContext(FlowNodeContext);
  const agent = ctx?.agentsById.get(data.agentId);
  const isRoot = ctx?.rootNodeId === id;
  const isMissing = !agent;
  const status = ctx?.nodeStatuses?.[id];
  const badge = status ? STATUS_BADGE[status] : null;

  return (
    <div
      className="agent-node"
      aria-label={agent ? `Agent node: ${agent.name}` : "Agent node: missing agent"}
      style={{
        border: isMissing ? "1px solid #b00020" : "1px solid #888",
        borderRadius: 6,
        padding: 8,
        background: "#fff",
        minWidth: 160,
      }}
    >
      <Handle type="target" position={Position.Left} />

      <div style={{ display: "flex", justifyContent: "space-between", gap: 8 }}>
        {isMissing ? (
          <span role="alert" style={{ color: "#b00020" }}>
            Agent missing
          </span>
        ) : (
          <span>
            <strong>{agent.name}</strong>
            <br />
            <small>{agent.model}</small>
          </span>
        )}
        {isRoot && <span aria-label="Root Agent">Root</span>}
      </div>

      {badge && (
        <div
          aria-label={`Status: ${badge.label}`}
          style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 6 }}
        >
          <span
            style={{
              width: 8,
              height: 8,
              borderRadius: "50%",
              background: badge.color,
              display: "inline-block",
            }}
          />
          <small style={{ color: badge.color }}>{badge.label}</small>
        </div>
      )}

      <div style={{ display: "flex", gap: 4, marginTop: 6 }}>
        <button
          type="button"
          aria-pressed={isRoot}
          onClick={() => ctx?.onSetRoot(id)}
        >
          {isRoot ? "Root" : "Set root"}
        </button>
        <button type="button" onClick={() => ctx?.onDuplicate(id)}>
          Duplicate
        </button>
        <button type="button" onClick={() => ctx?.onDetach(id)}>
          Detach
        </button>
        <button type="button" onClick={() => ctx?.onDelete(id)}>
          Delete
        </button>
      </div>

      <Handle type="source" position={Position.Right} />
    </div>
  );
}
