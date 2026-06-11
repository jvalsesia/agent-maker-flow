import { useCallback, useMemo, useState } from "react";
import { ReactFlowProvider, type Connection } from "@xyflow/react";
import { useQueryClient } from "@tanstack/react-query";

import { useAgents, type Agent } from "../lib/agents";
import * as ops from "../lib/flowGraph";
import type { FlowGraph } from "../lib/flowGraph";
import { useFlowGraph } from "../lib/useFlowGraph";
import { apiGet, ApiClientError } from "../lib/apiClient";
import {
  flowQueryKey,
  useCreateFlow,
  useDeleteFlow,
  useFlows,
  useRenameFlow,
  useUpdateFlow,
  type Flow,
  type FlowSummary,
} from "../lib/flows";
import { AgentPalette } from "../components/flow/AgentPalette";
import { FlowCanvas } from "../components/flow/FlowCanvas";
import { FlowToolbar } from "../components/flow/FlowToolbar";
import { SavedFlowsList } from "../components/flow/SavedFlowsList";
import { SaveFlowDialog } from "../components/flow/SaveFlowDialog";
import { DeleteFlowDialog } from "../components/flow/DeleteFlowDialog";
import { MissingAgentsBanner } from "../components/flow/MissingAgentsBanner";
import type { FlowNodeContextValue } from "../components/flow/AgentNode";

/**
 * Stable fingerprint of the persistence-relevant graph state (ignores React
 * Flow's transient selection so a mere click is not "dirty"). Used to drive the
 * unsaved-changes guard and the saved/unsaved indicator.
 */
function fingerprint(graph: FlowGraph): string {
  return JSON.stringify({
    nodes: graph.nodes.map((n) => ({
      id: n.id,
      type: n.type,
      position: n.position,
      data: n.data,
    })),
    edges: graph.edges.map((e) => ({
      id: e.id,
      source: e.source,
      target: e.target,
      sourceHandle: e.sourceHandle ?? null,
      targetHandle: e.targetHandle ?? null,
    })),
    rootNodeId: graph.rootNodeId,
  });
}

type SaveDialogState =
  | { mode: "save" }
  | { mode: "rename"; flow: FlowSummary }
  | null;

/**
 * Flows workspace (F07 canvas + F08 persistence): composes the saved-flows
 * list, the agent palette, the React Flow canvas, the toolbar, and the
 * save/rename/delete dialogs. It owns the graph via `useFlowGraph`, tracks a
 * saved snapshot to detect unsaved changes, guards opening/clearing over a
 * dirty canvas, and flags nodes whose referenced agent was deleted.
 */
export function FlowsPage() {
  const { data: agents } = useAgents();
  const { data: flows } = useFlows();
  const queryClient = useQueryClient();
  const flow = useFlowGraph();

  const createFlow = useCreateFlow();
  const updateFlow = useUpdateFlow();
  const renameFlow = useRenameFlow();
  const deleteFlow = useDeleteFlow();

  const [activeFlowId, setActiveFlowId] = useState<string | null>(null);
  const [activeName, setActiveName] = useState<string | null>(null);
  const [savedFingerprint, setSavedFingerprint] = useState(() => fingerprint(ops.emptyGraph()));

  const [connectionError, setConnectionError] = useState<string | null>(null);
  const [saveDialog, setSaveDialog] = useState<SaveDialogState>(null);
  const [dialogError, setDialogError] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<FlowSummary | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [pendingAction, setPendingAction] = useState<{ run: () => void } | null>(null);

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

  const isDirty = fingerprint(flow.graph) !== savedFingerprint;

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

  /** Replace the canvas with a saved flow, fetching its full graph on demand. */
  const openFlow = useCallback(
    async (summary: FlowSummary) => {
      const full = await queryClient.fetchQuery({
        queryKey: flowQueryKey(summary.id),
        queryFn: () => apiGet<Flow>(`/flows/${summary.id}`),
      });
      flow.load(full.graph);
      setActiveFlowId(full.id);
      setActiveName(full.name);
      setSavedFingerprint(fingerprint(full.graph));
      setSaveError(null);
    },
    [queryClient, flow],
  );

  /** Run an action immediately, or hold it behind the unsaved-changes guard. */
  const guarded = useCallback(
    (action: () => void) => {
      if (isDirty) setPendingAction({ run: action });
      else action();
    },
    [isDirty],
  );

  const handleOpen = useCallback(
    (summary: FlowSummary) => guarded(() => void openFlow(summary)),
    [guarded, openFlow],
  );

  const handleNew = useCallback(
    () =>
      guarded(() => {
        flow.load(ops.emptyGraph());
        setActiveFlowId(null);
        setActiveName(null);
        setSavedFingerprint(fingerprint(ops.emptyGraph()));
        setSaveError(null);
      }),
    [guarded, flow],
  );

  /** Save: update the open flow if there is one, otherwise prompt for a name. */
  const handleSave = useCallback(async () => {
    if (activeFlowId && activeName) {
      try {
        const saved = await updateFlow.mutateAsync({
          id: activeFlowId,
          input: { name: activeName, graph: flow.graph },
        });
        setSavedFingerprint(fingerprint(saved.graph));
        setSaveError(null);
      } catch (err) {
        // The save failed; the local canvas state is preserved untouched.
        setSaveError(err instanceof ApiClientError ? err.message : "Failed to save the flow.");
      }
      return;
    }
    setDialogError(null);
    setSaveDialog({ mode: "save" });
  }, [activeFlowId, activeName, updateFlow, flow.graph]);

  const handleSaveAs = useCallback(() => {
    setDialogError(null);
    setSaveDialog({ mode: "save" });
  }, []);

  const handleSaveDialogSubmit = useCallback(
    async (name: string) => {
      try {
        if (saveDialog?.mode === "rename") {
          const flowId = saveDialog.flow.id;
          await renameFlow.mutateAsync({ id: flowId, name });
          if (flowId === activeFlowId) setActiveName(name);
        } else {
          const saved = await createFlow.mutateAsync({ name, graph: flow.graph });
          setActiveFlowId(saved.id);
          setActiveName(saved.name);
          setSavedFingerprint(fingerprint(saved.graph));
        }
        setSaveDialog(null);
        setDialogError(null);
        setSaveError(null);
      } catch (err) {
        setDialogError(
          err instanceof ApiClientError ? err.message : "Could not save the flow.",
        );
      }
    },
    [saveDialog, renameFlow, createFlow, flow.graph, activeFlowId],
  );

  const handleDeleteConfirm = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await deleteFlow.mutateAsync(deleteTarget.id);
      if (deleteTarget.id === activeFlowId) {
        setActiveFlowId(null);
        setActiveName(null);
      }
      setDeleteTarget(null);
      setDeleteError(null);
    } catch (err) {
      setDeleteError(
        err instanceof ApiClientError ? err.message : "Could not delete the flow.",
      );
    }
  }, [deleteTarget, deleteFlow, activeFlowId]);

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

      <div role="toolbar" aria-label="Flow file actions" style={{ display: "flex", gap: 8 }}>
        <button type="button" onClick={handleNew}>
          New
        </button>
        <button type="button" onClick={() => void handleSave()}>
          Save
        </button>
        <button type="button" onClick={handleSaveAs}>
          Save As
        </button>
        <span role="status">
          {activeName ? `${activeName}${isDirty ? " — unsaved changes" : ""}` : "Unsaved flow"}
        </span>
      </div>

      {saveError && <p role="alert">{saveError}</p>}
      {connectionError && <p role="alert">{connectionError}</p>}
      <MissingAgentsBanner missingAgentIds={missingIds} />

      <div style={{ display: "flex", gap: 12, marginTop: 12 }}>
        <aside aria-label="Saved flows" style={{ width: 260 }}>
          <SavedFlowsList
            flows={flows ?? []}
            activeFlowId={activeFlowId}
            onOpen={handleOpen}
            onRename={(f) => {
              setDialogError(null);
              setSaveDialog({ mode: "rename", flow: f });
            }}
            onDelete={(f) => {
              setDeleteError(null);
              setDeleteTarget(f);
            }}
          />
        </aside>

        <ReactFlowProvider>
          <div style={{ flex: 1 }}>
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
          </div>
        </ReactFlowProvider>
      </div>

      {saveDialog && (
        <SaveFlowDialog
          mode={saveDialog.mode}
          initialName={saveDialog.mode === "rename" ? saveDialog.flow.name : ""}
          onSubmit={(name) => void handleSaveDialogSubmit(name)}
          onCancel={() => {
            setSaveDialog(null);
            setDialogError(null);
          }}
          isSubmitting={createFlow.isPending || renameFlow.isPending}
          error={dialogError}
        />
      )}

      {deleteTarget && (
        <DeleteFlowDialog
          flowName={deleteTarget.name}
          onConfirm={() => void handleDeleteConfirm()}
          onCancel={() => {
            setDeleteTarget(null);
            setDeleteError(null);
          }}
          isDeleting={deleteFlow.isPending}
          error={deleteError}
        />
      )}

      {pendingAction && (
        <div role="dialog" aria-modal="true" aria-label="Unsaved changes">
          <h3>Discard unsaved changes?</h3>
          <p>The canvas has unsaved changes that will be lost. Continue?</p>
          <div>
            <button
              type="button"
              onClick={() => {
                pendingAction.run();
                setPendingAction(null);
              }}
            >
              Discard and continue
            </button>
            <button type="button" onClick={() => setPendingAction(null)}>
              Cancel
            </button>
          </div>
        </div>
      )}
    </section>
  );
}
