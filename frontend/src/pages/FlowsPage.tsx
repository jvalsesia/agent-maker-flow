import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ReactFlowProvider, type Connection } from "@xyflow/react";
import { useQueryClient } from "@tanstack/react-query";

import { useAgents, type Agent } from "../lib/agents";
import * as ops from "../lib/flowGraph";
import type { FlowGraph } from "../lib/flowGraph";
import { useFlowGraph } from "../lib/useFlowGraph";
import { apiGet, ApiClientError } from "../lib/apiClient";
import { startRun } from "../lib/runs";
import { EMPTY_OUTPUT_MESSAGE, nodeStatusMap } from "../lib/runStream";
import { useRunStream } from "../hooks/useRunStream";
import { ConversationMonitor } from "../components/monitor/ConversationMonitor";
import type { ConversationTurn } from "../components/monitor/ConversationTurns";
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
import { Alert, Button, Card, Modal, useToast } from "../components/ui";
import styles from "./FlowsPage.module.css";

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
  const { showToast } = useToast();

  const [activeFlowId, setActiveFlowId] = useState<string | null>(null);
  const [activeName, setActiveName] = useState<string | null>(null);
  const [savedFingerprint, setSavedFingerprint] = useState(() => fingerprint(ops.emptyGraph()));

  const [saveDialog, setSaveDialog] = useState<SaveDialogState>(null);
  const [dialogError, setDialogError] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<FlowSummary | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [pendingAction, setPendingAction] = useState<{ run: () => void } | null>(null);
  // Monitor drawer (open state only matters below the three-pane width).
  const [monitorOpen, setMonitorOpen] = useState(false);
  // Saved-flows browser, now surfaced in a modal rather than the sidebar.
  const [flowsModalOpen, setFlowsModalOpen] = useState(false);

  // F10 run state: the current prompt, the active run id, and the session turns.
  const [prompt, setPrompt] = useState("");
  const [runId, setRunId] = useState<string | null>(null);
  const [turns, setTurns] = useState<ConversationTurn[]>([]);
  const finishedRunRef = useRef<string | null>(null);

  const { state: runState, connection } = useRunStream(runId);
  const isRunning = runId !== null && runState.status === "running";
  const nodeStatuses = useMemo(() => nodeStatusMap(runState), [runState]);
  const runNodes = useMemo(() => Object.values(runState.nodes), [runState.nodes]);

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

  // Why the run control is blocked (null = runnable), mirroring FlowToolbar.
  const runDisabledReason =
    flow.nodes.length === 0
      ? "Add an agent to the canvas before running."
      : flow.rootNodeId === null
        ? "Assign a Root Agent before running."
        : missingIds.length > 0
          ? "Resolve missing agents before running."
          : null;

  /** Dispatch a run for the live canvas; map a rejection to a system turn. */
  const handleRun = useCallback(
    async (submitted: string) => {
      const text = submitted.trim();
      if (!text || isRunning || runDisabledReason) return;

      const stamp = `${flow.nodes.length}-${turns.length}`;
      setTurns((prev) => [...prev, { id: `user-${stamp}`, role: "user", content: text }]);
      setPrompt("");

      try {
        const accepted = await startRun({
          prompt: text,
          graph: flow.graph,
          flowId: activeFlowId ?? undefined,
        });
        finishedRunRef.current = null;
        setRunId(accepted.runId);
      } catch (err) {
        const message =
          err instanceof ApiClientError ? err.message : "Could not start the run. Please retry.";
        setTurns((prev) => [...prev, { id: `system-${stamp}`, role: "system", content: message }]);
        setRunId(null);
      }
    },
    [isRunning, runDisabledReason, flow.nodes.length, flow.graph, turns.length, activeFlowId],
  );

  // When the active run reaches a terminal state, append the assistant turn
  // (the aggregated output, or the empty-output notice), exactly once per run.
  useEffect(() => {
    if (!runId) return;
    if (runState.status !== "succeeded" && runState.status !== "failed") return;
    if (finishedRunRef.current === runId) return;
    finishedRunRef.current = runId;
    const content =
      runState.output && runState.output.trim().length > 0
        ? runState.output
        : EMPTY_OUTPUT_MESSAGE;
    setTurns((prev) => [...prev, { id: `assistant-${runId}`, role: "assistant", content }]);
  }, [runId, runState.status, runState.output]);

  const handleConnect = useCallback(
    (connection: Connection) => {
      const added = flow.connect(connection);
      if (!added) {
        showToast(
          "Connection rejected: flows must stay acyclic — no self-loops, cycles, or duplicate edges.",
          "error",
        );
      }
    },
    [flow, showToast],
  );

  /** Keyboard fallback for palette drag-and-drop: drop the agent at a cascading
   *  default position so it never lands exactly on the previous node (spec §7). */
  const handleAddToCanvas = useCallback(
    (agentId: string) => {
      const n = flow.nodes.length;
      flow.addAgentNode(agentId, { x: 80 + (n % 5) * 48, y: 80 + (n % 5) * 48 });
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
        showToast("Flow saved", "success");
      } catch (err) {
        // The save failed; the local canvas state is preserved untouched.
        setSaveError(err instanceof ApiClientError ? err.message : "Failed to save the flow.");
      }
      return;
    }
    setDialogError(null);
    setSaveDialog({ mode: "save" });
  }, [activeFlowId, activeName, updateFlow, flow.graph, showToast]);

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
          showToast("Flow renamed", "success");
        } else {
          const saved = await createFlow.mutateAsync({ name, graph: flow.graph });
          setActiveFlowId(saved.id);
          setActiveName(saved.name);
          setSavedFingerprint(fingerprint(saved.graph));
          showToast("Flow saved", "success");
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
    [saveDialog, renameFlow, createFlow, flow.graph, activeFlowId, showToast],
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
      showToast("Flow deleted", "success");
    } catch (err) {
      setDeleteError(
        err instanceof ApiClientError ? err.message : "Could not delete the flow.",
      );
    }
  }, [deleteTarget, deleteFlow, activeFlowId, showToast]);

  const context = useMemo<FlowNodeContextValue>(
    () => ({
      agentsById,
      rootNodeId: flow.rootNodeId,
      nodeStatuses,
      onSetRoot: flow.setRoot,
      onDuplicate: flow.duplicateNode,
      onDetach: flow.detachNode,
      onDelete: flow.removeNode,
    }),
    [
      agentsById,
      flow.rootNodeId,
      nodeStatuses,
      flow.setRoot,
      flow.duplicateNode,
      flow.detachNode,
      flow.removeNode,
    ],
  );

  return (
    <section className={styles.page} aria-label="Flows workspace">
      <a className="skip-link" href="#flow-monitor">
        Skip to monitor
      </a>
      <div className={styles.toolbar} role="toolbar" aria-label="Flow file actions">
        <div className={styles.toolbarLeft}>
          <h2 className={styles.heading}>Flows</h2>
          <span className={styles.divider} aria-hidden="true" />
          <span className={styles.flowName}>{activeName ?? "Untitled flow"}</span>
          {isDirty && (
            <span className={styles.dirty} role="status">
              <span className={styles.dirtyDot} aria-hidden="true" />
              Unsaved
            </span>
          )}
        </div>
        <div className={styles.toolbarRight}>
          <span className={styles.status}>
            {activeName ? (isDirty ? "Unsaved changes" : "Saved") : "Not saved yet"}
          </span>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => setFlowsModalOpen(true)}
            aria-haspopup="dialog"
          >
            Open flow{flows && flows.length > 0 ? ` (${flows.length})` : ""}
          </Button>
          <Button variant="secondary" size="sm" onClick={handleNew}>
            New
          </Button>
          <Button variant="secondary" size="sm" onClick={() => void handleSave()}>
            Save
          </Button>
          <Button variant="secondary" size="sm" onClick={handleSaveAs}>
            Save as
          </Button>
          <span className={styles.monitorToggle}>
            <Button
              variant="secondary"
              size="sm"
              aria-expanded={monitorOpen}
              aria-controls="flow-monitor"
              onClick={() => setMonitorOpen((open) => !open)}
            >
              Monitor
            </Button>
          </span>
        </div>
      </div>

      {(saveError || missingIds.length > 0) && (
        <div className={styles.banners}>
          {saveError && (
            <Alert variant="danger" role="alert">
              {saveError}
            </Alert>
          )}
          <MissingAgentsBanner missingAgentIds={missingIds} />
        </div>
      )}

      <div className={styles.workspace}>
        <aside className={styles.left} aria-label="Agents palette">
          <Card title="Agents palette" className={styles.paletteCard} bodyClassName={styles.scrollBody}>
            <AgentPalette onAddToCanvas={handleAddToCanvas} />
          </Card>
        </aside>

        <ReactFlowProvider>
          <div className={styles.center}>
            <p className={styles.mobileNotice} role="note">
              Flow editing works best on a larger screen. Drag-and-drop is desktop-first; use a
              wider viewport to build flows.
            </p>
            <div className={styles.canvasWrap}>
              <div className={styles.runOverlay}>
                <FlowToolbar
                  hasNodes={flow.nodes.length > 0}
                  hasRoot={flow.rootNodeId !== null}
                  missingAgentCount={missingIds.length}
                  isRunning={isRunning}
                  onRun={() => void handleRun(prompt)}
                />
              </div>
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

        {monitorOpen && (
          <div
            className={styles.scrim}
            aria-hidden="true"
            onClick={() => setMonitorOpen(false)}
          />
        )}
        <aside
          id="flow-monitor"
          tabIndex={-1}
          className={[styles.right, monitorOpen && styles.drawerOpen].filter(Boolean).join(" ")}
          aria-label="Conversation monitor panel"
        >
          <Card className={styles.monitorCard}>
            <ConversationMonitor
              turns={turns}
              nodes={runNodes}
              connection={connection}
              isRunning={isRunning}
              prompt={prompt}
              onPromptChange={setPrompt}
              onSubmit={(submitted) => void handleRun(submitted)}
              runDisabledReason={runDisabledReason}
            />
          </Card>
        </aside>
      </div>

      <Modal
        open={flowsModalOpen}
        onClose={() => setFlowsModalOpen(false)}
        title="Saved flows"
        size="lg"
        footer={
          <Button variant="secondary" onClick={() => setFlowsModalOpen(false)}>
            Close
          </Button>
        }
      >
        <SavedFlowsList
          flows={flows ?? []}
          activeFlowId={activeFlowId}
          onOpen={(f) => {
            // Close the browser first so the unsaved-changes guard (if any)
            // isn't stacked on top of this modal.
            setFlowsModalOpen(false);
            handleOpen(f);
          }}
          onRename={(f) => {
            setFlowsModalOpen(false);
            setDialogError(null);
            setSaveDialog({ mode: "rename", flow: f });
          }}
          onDelete={(f) => {
            setFlowsModalOpen(false);
            setDeleteError(null);
            setDeleteTarget(f);
          }}
        />
      </Modal>

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
        <Modal
          open
          onClose={() => setPendingAction(null)}
          title="Discard unsaved changes?"
          size="sm"
          footer={
            <>
              <Button variant="secondary" onClick={() => setPendingAction(null)}>
                Cancel
              </Button>
              <Button
                variant="danger"
                onClick={() => {
                  pendingAction.run();
                  setPendingAction(null);
                }}
              >
                Discard and continue
              </Button>
            </>
          }
        >
          <p style={{ margin: 0 }}>The canvas has unsaved changes that will be lost. Continue?</p>
        </Modal>
      )}
    </section>
  );
}
