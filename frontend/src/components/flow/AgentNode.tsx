import { createContext, useContext } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";

import type { Agent } from "../../lib/agents";
import type { FlowNode } from "../../lib/flowGraph";

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
