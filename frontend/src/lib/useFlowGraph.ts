import { useCallback, useMemo, useState } from "react";
import {
  useEdgesState,
  useNodesState,
  type Connection,
  type OnEdgesChange,
  type OnNodesChange,
} from "@xyflow/react";

import * as ops from "./flowGraph";
import type { FlowEdge, FlowGraph, FlowNode } from "./flowGraph";

/** Everything the Flows page needs to drive the canvas and its invariants. */
export interface UseFlowGraph {
  nodes: FlowNode[];
  edges: FlowEdge[];
  rootNodeId: string | null;
  /** The derived serializable graph (the seam for F08 persistence / F09 run). */
  graph: FlowGraph;
  onNodesChange: OnNodesChange<FlowNode>;
  onEdgesChange: OnEdgesChange<FlowEdge>;
  /** Validated connect; returns false (and draws nothing) if the edge is invalid. */
  connect: (connection: Connection) => boolean;
  addAgentNode: (agentId: string, position: { x: number; y: number }) => void;
  removeNode: (nodeId: string) => void;
  duplicateNode: (nodeId: string) => void;
  detachNode: (nodeId: string) => void;
  setRoot: (nodeId: string) => void;
  /** Replace the whole canvas with a saved graph (F08 open). */
  load: (graph: FlowGraph) => void;
}

/**
 * Wraps React Flow's node/edge state with the F07 domain invariants — single
 * root, validated connections (no self-loops, duplicates, or cycles), and the
 * node operations — delegating the logic to the pure helpers in `flowGraph`.
 * Position/selection changes flow through React Flow's own change handlers.
 */
export function useFlowGraph(initial?: FlowGraph): UseFlowGraph {
  const [nodes, setNodes, onNodesChange] = useNodesState<FlowNode>(initial?.nodes ?? []);
  const [edges, setEdges, onEdgesChange] = useEdgesState<FlowEdge>(initial?.edges ?? []);
  const [rootNodeId, setRootNodeId] = useState<string | null>(initial?.rootNodeId ?? null);

  const graph = useMemo<FlowGraph>(
    () => ({ nodes, edges, rootNodeId }),
    [nodes, edges, rootNodeId],
  );

  const connect = useCallback(
    (connection: Connection): boolean => {
      const current: FlowGraph = { nodes, edges, rootNodeId };
      if (!ops.isValidConnection(current, connection)) return false;
      setEdges(ops.connect(current, connection).edges);
      return true;
    },
    [nodes, edges, rootNodeId, setEdges],
  );

  const addAgentNode = useCallback(
    (agentId: string, position: { x: number; y: number }) => {
      const next = ops.addAgentNode({ nodes, edges, rootNodeId }, agentId, position);
      setNodes(next.nodes);
    },
    [nodes, edges, rootNodeId, setNodes],
  );

  const removeNode = useCallback(
    (nodeId: string) => {
      const next = ops.removeNode({ nodes, edges, rootNodeId }, nodeId);
      setNodes(next.nodes);
      setEdges(next.edges);
      setRootNodeId(next.rootNodeId);
    },
    [nodes, edges, rootNodeId, setNodes, setEdges],
  );

  const duplicateNode = useCallback(
    (nodeId: string) => {
      const next = ops.duplicateNode({ nodes, edges, rootNodeId }, nodeId);
      setNodes(next.nodes);
    },
    [nodes, edges, rootNodeId, setNodes],
  );

  const detachNode = useCallback(
    (nodeId: string) => {
      const next = ops.detachNode({ nodes, edges, rootNodeId }, nodeId);
      setEdges(next.edges);
    },
    [nodes, edges, rootNodeId, setEdges],
  );

  const setRoot = useCallback((nodeId: string) => {
    setRootNodeId(nodeId);
  }, []);

  const load = useCallback(
    (next: FlowGraph) => {
      setNodes(next.nodes);
      setEdges(next.edges);
      setRootNodeId(next.rootNodeId);
    },
    [setNodes, setEdges],
  );

  return {
    nodes,
    edges,
    rootNodeId,
    graph,
    onNodesChange,
    onEdgesChange,
    connect,
    addAgentNode,
    removeNode,
    duplicateNode,
    detachNode,
    setRoot,
    load,
  };
}
