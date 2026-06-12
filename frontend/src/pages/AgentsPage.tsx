import { useState } from "react";

import {
  useAgents,
  useCreateAgent,
  useDeleteAgent,
  useUpdateAgent,
  type Agent,
  type AgentInput,
} from "../lib/agents";
import { AgentForm, type AgentFormMode } from "../components/agents/AgentForm";
import { AgentList } from "../components/agents/AgentList";
import { DeleteAgentDialog } from "../components/agents/DeleteAgentDialog";
import { Alert, Button, Card, SkeletonRows, useToast } from "../components/ui";
import styles from "./AgentsPage.module.css";

type Editor =
  | { mode: "create" }
  | { mode: "edit"; agent: Agent }
  | { mode: "duplicate"; agent: Agent }
  | null;

/**
 * Agents Dashboard (F04): the registry list plus the create/edit/duplicate form
 * and the delete confirmation. Reads/writes go through the agents hooks, which
 * invalidate the list on success.
 */
export function AgentsPage() {
  const agentsQuery = useAgents();
  const createAgent = useCreateAgent();
  const updateAgent = useUpdateAgent();
  const deleteAgent = useDeleteAgent();
  const { showToast } = useToast();

  const [editor, setEditor] = useState<Editor>(null);
  const [deleteTarget, setDeleteTarget] = useState<Agent | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  async function handleSubmit(input: AgentInput) {
    if (editor?.mode === "edit") {
      await updateAgent.mutateAsync({ id: editor.agent.id, input });
      showToast("Agent saved", "success");
    } else {
      await createAgent.mutateAsync(input);
      showToast(editor?.mode === "duplicate" ? "Agent duplicated" : "Agent created", "success");
    }
    setEditor(null);
  }

  async function handleConfirmDelete() {
    if (!deleteTarget) return;
    setDeleteError(null);
    try {
      await deleteAgent.mutateAsync(deleteTarget.id);
      setDeleteTarget(null);
      showToast("Agent deleted", "success");
    } catch {
      setDeleteError("Could not delete agent. Please retry.");
    }
  }

  const editorMode: AgentFormMode | undefined = editor?.mode;
  const editorInitial = editor && editor.mode !== "create" ? editor.agent : undefined;

  return (
    <section className={styles.page} aria-label="Agents workspace">
      <header className={styles.header}>
        <h2 className={styles.title}>Agents</h2>
        <Button
          variant="primary"
          onClick={() => setEditor({ mode: "create" })}
          leadingIcon={
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
              <path strokeLinecap="round" d="M12 5v14M5 12h14" />
            </svg>
          }
        >
          New agent
        </Button>
      </header>

      {agentsQuery.isLoading && (
        <Card>
          <SkeletonRows count={5} />
        </Card>
      )}
      {agentsQuery.isError && (
        <Alert
          variant="danger"
          role="alert"
          title="Could not load agents"
          actions={
            <Button variant="secondary" size="sm" onClick={() => void agentsQuery.refetch()}>
              Retry
            </Button>
          }
        >
          Please retry.
        </Alert>
      )}

      {agentsQuery.data && (
        <AgentList
          agents={agentsQuery.data}
          onCreate={() => setEditor({ mode: "create" })}
          onEdit={(agent) => setEditor({ mode: "edit", agent })}
          onDuplicate={(agent) => setEditor({ mode: "duplicate", agent })}
          onDelete={(agent) => {
            setDeleteError(null);
            setDeleteTarget(agent);
          }}
        />
      )}

      {editorMode && (
        <AgentForm
          mode={editorMode}
          initial={editorInitial}
          onSubmit={handleSubmit}
          onCancel={() => setEditor(null)}
        />
      )}

      {deleteTarget && (
        <DeleteAgentDialog
          agent={deleteTarget}
          onConfirm={handleConfirmDelete}
          onCancel={() => setDeleteTarget(null)}
          isDeleting={deleteAgent.isPending}
          error={deleteError}
        />
      )}
    </section>
  );
}
