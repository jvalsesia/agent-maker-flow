import { useCallback, useMemo, useState } from "react";
import { ReactFlowProvider, type Connection } from "@xyflow/react";

import { useAgents, type Agent } from "../lib/agents";
import * as ops from "../lib/flowGraph";
import { useFlowGraph } from "../lib/useFlowGraph";
import { AgentPalette } from "../components/flow/AgentPalette";
import { FlowCanvas } from "../components/flow/FlowCanvas";
import { FlowToolbar } from "../components/flow/FlowToolbar";
import type { FlowNodeContextValue } from "../components/flow/AgentNode";

/**
 * Flows workspace (F07): composes the agent palette, the React Flow canvas, and
 * the floating toolbar. It owns the graph state via `useFlowGraph`, surfaces the
 * inline rejection message for invalid connections, and flags nodes whose
 * referenced agent was deleted from the registry (which blocks running). The
 * graph itself is the serializable seam that F08 will persist and F09 execute.
 */
export function FlowsPage() {
  const { data: agents } = useAgents();
  const flow = useFlowGraph();
  const [connectionError, setConnectionError] = useState<string | null>(null);

  const agentsById = useMemo(() => {
    const map = new Map<string, Agent>();
    for (const agent of agents ?? []) map.set(agent.id, agent);
    return map;
  }, [agents]);

  const availableAgentIds = useMemo(() => (agents ?? []).map((a) => a.id), [agents]);
  const missingIds = useMemo(
    () => ops.missingAgentIds(flow.graph, availableAgentIds),
    [flow.graph, availableAgentIds],
  );

  const handleConnect = useCallback(
    (connection: Connection) => {
      const added = flow.connect(connection);
      setConnectionError(
        added
          ? null
          : "Connection rejected: a flow must stay acyclic — no self-loops, cycles, or duplicate edges.",
      );
    },
    [flow],
  );

  const context = useMemo<FlowNodeContextValue>(
    () => ({
      agentsById,
      rootNodeId: flow.rootNodeId,
      onSetRoot: flow.setRoot,
      onDuplicate: flow.duplicateNode,
      onDetach: flow.detachNode,
      onDelete: flow.removeNode,
    }),
    [agentsById, flow.rootNodeId, flow.setRoot, flow.duplicateNode, flow.detachNode, flow.removeNode],
  );

  return (
    <section aria-label="Flows workspace">
      <h2>Flows</h2>

      {connectionError && <p role="alert">{connectionError}</p>}
      {missingIds.length > 0 && (
        <p role="alert">
          {missingIds.length} node(s) reference an agent that no longer exists. Remove or repoint
          them before running.
        </p>
      )}

      <ReactFlowProvider>
        <FlowToolbar
          hasNodes={flow.nodes.length > 0}
          hasRoot={flow.rootNodeId !== null}
          missingAgentCount={missingIds.length}
        />

        <div style={{ display: "flex", gap: 12, marginTop: 12 }}>
          <AgentPalette />
          <div style={{ flex: 1, height: 600, border: "1px solid #ddd", borderRadius: 6 }}>
            <FlowCanvas
              nodes={flow.nodes}
              edges={flow.edges}
              rootNodeId={flow.rootNodeId}
              onNodesChange={flow.onNodesChange}
              onEdgesChange={flow.onEdgesChange}
              onConnect={handleConnect}
              onDropAgent={flow.addAgentNode}
              context={context}
            />
          </div>
        </div>
      </ReactFlowProvider>
    </section>
  );
}
