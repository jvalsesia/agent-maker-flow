import type { Connection, Edge, Node } from "@xyflow/react";

/**
 * The serializable flow graph model (F07). It is intentionally plain — agent
 * references by id, node positions, and edge endpoints — so F08 can persist and
 * rehydrate it without transformation and F09 can translate it into an
 * execution DAG. F07 itself adds no backend; this model is the seam.
 */

/** Per-node data: a reference to an F04 agent by id. */
export interface FlowNodeData {
  agentId: string;
  [key: string]: unknown;
}

/** A canvas node referencing an agent. */
export type FlowNode = Node<FlowNodeData>;

/** A forwarding edge (output port → input port). */
export type FlowEdge = Edge;

/** The whole graph plus the single Root Agent assignment. */
export interface FlowGraph {
  nodes: FlowNode[];
  edges: FlowEdge[];
  /** The single Root Agent; null until one is assigned. */
  rootNodeId: string | null;
}

/** An empty graph. */
export function emptyGraph(): FlowGraph {
  return { nodes: [], edges: [], rootNodeId: null };
}

/**
 * Next collision-free node id (`node-1`, `node-2`, …), derived from the ids
 * already in the graph rather than a module-global counter. Deriving it from
 * the graph is what makes it safe after `load()` rehydrates a saved flow: a
 * fresh session's counter has no idea `node-1..3` already exist, so a global
 * counter would re-mint a colliding id. We take one past the highest existing
 * `node-N` suffix instead.
 */
function nextNodeId(graph: FlowGraph): string {
  let max = 0;
  for (const node of graph.nodes) {
    const match = /^node-(\d+)$/.exec(node.id);
    if (match) max = Math.max(max, Number(match[1]));
  }
  return `node-${max + 1}`;
}

/** Build a stable edge id from its endpoints. */
function edgeId(source: string, target: string): string {
  return `edge-${source}-${target}`;
}

/**
 * `true` if adding `source → target` would make the graph cyclic. A self-loop
 * (`source === target`) is treated as a cycle. Otherwise we look for an
 * existing path from `target` back to `source`: if one exists, the new edge
 * closes a cycle.
 */
export function wouldCreateCycle(
  edges: FlowEdge[],
  source: string,
  target: string,
): boolean {
  if (source === target) return true;

  // Adjacency: node -> nodes it points at.
  const out = new Map<string, string[]>();
  for (const edge of edges) {
    const list = out.get(edge.source);
    if (list) list.push(edge.target);
    else out.set(edge.source, [edge.target]);
  }

  // Does a path already exist from `target` to `source`?
  const stack = [target];
  const seen = new Set<string>();
  while (stack.length > 0) {
    const current = stack.pop() as string;
    if (current === source) return true;
    if (seen.has(current)) continue;
    seen.add(current);
    const neighbors = out.get(current);
    if (neighbors) stack.push(...neighbors);
  }
  return false;
}

/**
 * Whether a proposed connection may be drawn. Rejects when it lacks endpoints,
 * self-loops, duplicates an existing edge, or would create a cycle.
 */
export function isValidConnection(
  graph: FlowGraph,
  connection: Connection,
): boolean {
  const { source, target } = connection;
  if (!source || !target) return false;
  if (source === target) return false;
  const duplicate = graph.edges.some(
    (edge) => edge.source === source && edge.target === target,
  );
  if (duplicate) return false;
  return !wouldCreateCycle(graph.edges, source, target);
}

/** Append a node referencing `agentId` at `position`. Root is untouched. */
export function addAgentNode(
  graph: FlowGraph,
  agentId: string,
  position: { x: number; y: number },
): FlowGraph {
  const node: FlowNode = {
    id: nextNodeId(graph),
    type: "agent",
    position,
    data: { agentId },
  };
  return { ...graph, nodes: [...graph.nodes, node] };
}

/**
 * Add a validated connection. Returns the graph unchanged if the connection is
 * invalid (the caller surfaces the inline message); otherwise appends the edge.
 */
export function connect(graph: FlowGraph, connection: Connection): FlowGraph {
  if (!isValidConnection(graph, connection)) return graph;
  const { source, target } = connection;
  const edge: FlowEdge = {
    id: edgeId(source as string, target as string),
    source: source as string,
    target: target as string,
    sourceHandle: connection.sourceHandle ?? undefined,
    targetHandle: connection.targetHandle ?? undefined,
  };
  return { ...graph, edges: [...graph.edges, edge] };
}

/** Remove a node and every edge touching it; clear root if it was the root. */
export function removeNode(graph: FlowGraph, nodeId: string): FlowGraph {
  return {
    nodes: graph.nodes.filter((node) => node.id !== nodeId),
    edges: graph.edges.filter(
      (edge) => edge.source !== nodeId && edge.target !== nodeId,
    ),
    rootNodeId: graph.rootNodeId === nodeId ? null : graph.rootNodeId,
  };
}

/**
 * Duplicate a node: a fresh node with the same `agentId`, offset so it doesn't
 * overlap. It carries no edges and is never the root.
 */
export function duplicateNode(graph: FlowGraph, nodeId: string): FlowGraph {
  const source = graph.nodes.find((node) => node.id === nodeId);
  if (!source) return graph;
  const copy: FlowNode = {
    id: nextNodeId(graph),
    type: source.type,
    position: { x: source.position.x + 40, y: source.position.y + 40 },
    data: { agentId: source.data.agentId },
  };
  return { ...graph, nodes: [...graph.nodes, copy] };
}

/** Remove only the edges connected to a node; keep the node and the root. */
export function detachNode(graph: FlowGraph, nodeId: string): FlowGraph {
  return {
    ...graph,
    edges: graph.edges.filter(
      (edge) => edge.source !== nodeId && edge.target !== nodeId,
    ),
  };
}

/** Set the single Root Agent, replacing any previous root. */
export function setRoot(graph: FlowGraph, nodeId: string): FlowGraph {
  return { ...graph, rootNodeId: nodeId };
}

/** Run is allowed once a root is assigned and at least one node exists. */
export function canRun(graph: FlowGraph): boolean {
  return graph.rootNodeId !== null && graph.nodes.length > 0;
}

/**
 * Node-referenced agent ids that are absent from the registry (deduplicated).
 * A non-empty result means one or more nodes reference a deleted agent.
 */
export function missingAgentIds(
  graph: FlowGraph,
  availableAgentIds: string[],
): string[] {
  const available = new Set(availableAgentIds);
  const missing = new Set<string>();
  for (const node of graph.nodes) {
    if (!available.has(node.data.agentId)) missing.add(node.data.agentId);
  }
  return [...missing];
}
