import { useCallback, type DragEvent } from "react";
import {
  Background,
  Controls,
  ReactFlow,
  useReactFlow,
  type Connection,
  type Edge,
  type NodeTypes,
  type OnEdgesChange,
  type OnNodesChange,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import * as ops from "../../lib/flowGraph";
import type { FlowEdge, FlowNode } from "../../lib/flowGraph";
import { AgentNode, FlowNodeContext, type FlowNodeContextValue } from "./AgentNode";
import { AGENT_DRAG_MIME } from "./AgentPalette";

/** Custom node registry — `type: "agent"` maps to {@link AgentNode}. */
const nodeTypes: NodeTypes = { agent: AgentNode };

interface FlowCanvasProps {
  nodes: FlowNode[];
  edges: FlowEdge[];
  rootNodeId: string | null;
  onNodesChange: OnNodesChange<FlowNode>;
  onEdgesChange: OnEdgesChange<FlowEdge>;
  /** Called with a validated connection; the page decides what to do on reject. */
  onConnect: (connection: Connection) => void;
  /** Instantiate a node from an agent dropped onto the canvas. */
  onDropAgent: (agentId: string, position: { x: number; y: number }) => void;
  /** Per-node context (agent lookup, root, node-operation callbacks). */
  context: FlowNodeContextValue;
}

/**
 * The React Flow canvas (F07): renders the agent-node graph, accepts agents
 * dropped from the palette, and validates edge creation via the pure helpers so
 * self-loops and cycle-forming connections are rejected inline (no edge drawn).
 * Must be rendered inside a `ReactFlowProvider` (owned by the Flows page).
 */
export function FlowCanvas({
  nodes,
  edges,
  rootNodeId,
  onNodesChange,
  onEdgesChange,
  onConnect,
  onDropAgent,
  context,
}: FlowCanvasProps) {
  const { screenToFlowPosition } = useReactFlow();

  const isValidConnection = useCallback(
    (connection: Connection | Edge) =>
      ops.isValidConnection({ nodes, edges, rootNodeId }, connection as Connection),
    [nodes, edges, rootNodeId],
  );

  const handleDragOver = useCallback((event: DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const handleDrop = useCallback(
    (event: DragEvent) => {
      event.preventDefault();
      const agentId = event.dataTransfer.getData(AGENT_DRAG_MIME);
      if (!agentId) return;
      const position = screenToFlowPosition({ x: event.clientX, y: event.clientY });
      onDropAgent(agentId, position);
    },
    [screenToFlowPosition, onDropAgent],
  );

  return (
    <div
      style={{ width: "100%", height: "100%" }}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      <FlowNodeContext.Provider value={context}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          isValidConnection={isValidConnection}
          fitView
        >
          <Background />
          <Controls />
        </ReactFlow>
      </FlowNodeContext.Provider>
    </div>
  );
}
