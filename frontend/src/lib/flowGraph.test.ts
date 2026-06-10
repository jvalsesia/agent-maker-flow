import { describe, expect, it } from "vitest";
import type { Connection } from "@xyflow/react";

import {
  addAgentNode,
  canRun,
  connect,
  detachNode,
  duplicateNode,
  emptyGraph,
  isValidConnection,
  missingAgentIds,
  removeNode,
  setRoot,
  wouldCreateCycle,
  type FlowEdge,
  type FlowGraph,
} from "./flowGraph";

/** Build a graph from explicit node ids and edges for terse assertions. */
function graphOf(nodeIds: string[], edges: Array<[string, string]>): FlowGraph {
  return {
    nodes: nodeIds.map((id) => ({
      id,
      type: "agent",
      position: { x: 0, y: 0 },
      data: { agentId: `agent-${id}` },
    })),
    edges: edges.map(([source, target]) => ({
      id: `edge-${source}-${target}`,
      source,
      target,
    })),
    rootNodeId: null,
  };
}

describe("wouldCreateCycle", () => {
  it("rejects a self-loop", () => {
    expect(wouldCreateCycle([], "n", "n")).toBe(true);
  });

  it("rejects a direct cycle (A→B then B→A)", () => {
    const edges: FlowEdge[] = [{ id: "e", source: "A", target: "B" }];
    expect(wouldCreateCycle(edges, "B", "A")).toBe(true);
  });

  it("rejects an indirect cycle (A→B→C then C→A)", () => {
    const edges: FlowEdge[] = [
      { id: "e1", source: "A", target: "B" },
      { id: "e2", source: "B", target: "C" },
    ];
    expect(wouldCreateCycle(edges, "C", "A")).toBe(true);
  });

  it("accepts a valid forward edge", () => {
    const edges: FlowEdge[] = [{ id: "e1", source: "A", target: "B" }];
    expect(wouldCreateCycle(edges, "B", "C")).toBe(false);
  });
});

describe("isValidConnection", () => {
  it("rejects a self-loop", () => {
    const graph = graphOf(["A"], []);
    expect(isValidConnection(graph, { source: "A", target: "A", sourceHandle: null, targetHandle: null })).toBe(false);
  });

  it("rejects a duplicate edge", () => {
    const graph = graphOf(["A", "B"], [["A", "B"]]);
    expect(isValidConnection(graph, { source: "A", target: "B", sourceHandle: null, targetHandle: null })).toBe(false);
  });

  it("rejects a cycle-forming edge", () => {
    const graph = graphOf(["A", "B"], [["A", "B"]]);
    expect(isValidConnection(graph, { source: "B", target: "A", sourceHandle: null, targetHandle: null })).toBe(false);
  });

  it("accepts a valid forward edge", () => {
    const graph = graphOf(["A", "B"], []);
    expect(isValidConnection(graph, { source: "A", target: "B", sourceHandle: null, targetHandle: null })).toBe(true);
  });

  it("rejects a connection missing an endpoint", () => {
    const graph = graphOf(["A"], []);
    // React Flow may hand us a connection with no source (drop on empty canvas).
    const incomplete = { source: null, target: "A", sourceHandle: null, targetHandle: null } as unknown as Connection;
    expect(isValidConnection(graph, incomplete)).toBe(false);
  });
});

describe("addAgentNode", () => {
  it("appends a node referencing the agent at the given position", () => {
    const before = emptyGraph();
    const after = addAgentNode(before, "agent-42", { x: 10, y: 20 });
    expect(after.nodes).toHaveLength(1);
    expect(after.nodes[0].data.agentId).toBe("agent-42");
    expect(after.nodes[0].position).toEqual({ x: 10, y: 20 });
    expect(before.nodes).toHaveLength(0); // immutable
  });
});

describe("connect", () => {
  it("appends a validated edge", () => {
    const graph = graphOf(["A", "B"], []);
    const after = connect(graph, { source: "A", target: "B", sourceHandle: null, targetHandle: null });
    expect(after.edges).toHaveLength(1);
    expect(after.edges[0]).toMatchObject({ source: "A", target: "B" });
  });

  it("leaves the graph unchanged for an invalid connection", () => {
    const graph = graphOf(["A", "B"], [["A", "B"]]);
    const after = connect(graph, { source: "B", target: "A", sourceHandle: null, targetHandle: null });
    expect(after.edges).toHaveLength(1);
  });
});

describe("setRoot", () => {
  it("replaces the previous root", () => {
    let graph = graphOf(["A", "B"], []);
    graph = setRoot(graph, "A");
    expect(graph.rootNodeId).toBe("A");
    graph = setRoot(graph, "B");
    expect(graph.rootNodeId).toBe("B");
  });
});

describe("removeNode", () => {
  it("removes the node, its edges, and clears root when it was root", () => {
    let graph = graphOf(["A", "B", "C"], [["A", "B"], ["B", "C"]]);
    graph = setRoot(graph, "B");
    const after = removeNode(graph, "B");
    expect(after.nodes.map((n) => n.id)).toEqual(["A", "C"]);
    expect(after.edges).toHaveLength(0);
    expect(after.rootNodeId).toBeNull();
  });

  it("keeps root when a non-root node is removed", () => {
    let graph = graphOf(["A", "B"], [["A", "B"]]);
    graph = setRoot(graph, "A");
    const after = removeNode(graph, "B");
    expect(after.rootNodeId).toBe("A");
    expect(after.edges).toHaveLength(0);
  });
});

describe("detachNode", () => {
  it("keeps the node but drops its edges", () => {
    const graph = graphOf(["A", "B", "C"], [["A", "B"], ["B", "C"]]);
    const after = detachNode(graph, "B");
    expect(after.nodes.map((n) => n.id)).toEqual(["A", "B", "C"]);
    expect(after.edges).toHaveLength(0);
  });
});

describe("duplicateNode", () => {
  it("copies the agentId only — new node, no edges, not root", () => {
    let graph = graphOf(["A", "B"], [["A", "B"]]);
    graph = setRoot(graph, "A");
    const after = duplicateNode(graph, "A");
    expect(after.nodes).toHaveLength(3);
    const copy = after.nodes[after.nodes.length - 1];
    expect(copy.data.agentId).toBe("agent-A");
    expect(copy.id).not.toBe("A");
    expect(after.edges).toHaveLength(1); // original edge only
    expect(after.rootNodeId).toBe("A"); // copy is not root
  });
});

describe("canRun", () => {
  it("is false without a root", () => {
    const graph = graphOf(["A"], []);
    expect(canRun(graph)).toBe(false);
  });

  it("is false without nodes even if a root id lingers", () => {
    const graph: FlowGraph = { nodes: [], edges: [], rootNodeId: "A" };
    expect(canRun(graph)).toBe(false);
  });

  it("is true with a root and at least one node", () => {
    const graph = setRoot(graphOf(["A"], []), "A");
    expect(canRun(graph)).toBe(true);
  });
});

describe("missingAgentIds", () => {
  it("flags node-referenced agents absent from the registry", () => {
    const graph = graphOf(["A", "B"], []);
    // agentId is `agent-A`, `agent-B`; registry only has agent-A.
    expect(missingAgentIds(graph, ["agent-A"])).toEqual(["agent-B"]);
  });

  it("returns empty when all referenced agents exist", () => {
    const graph = graphOf(["A", "B"], []);
    expect(missingAgentIds(graph, ["agent-A", "agent-B"])).toEqual([]);
  });
});
