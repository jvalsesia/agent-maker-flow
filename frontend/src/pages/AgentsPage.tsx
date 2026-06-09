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

  const [editor, setEditor] = useState<Editor>(null);
  const [deleteTarget, setDeleteTarget] = useState<Agent | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  async function handleSubmit(input: AgentInput) {
    if (editor?.mode === "edit") {
      await updateAgent.mutateAsync({ id: editor.agent.id, input });
    } else {
      await createAgent.mutateAsync(input);
    }
    setEditor(null);
  }

  async function handleConfirmDelete() {
    if (!deleteTarget) return;
    setDeleteError(null);
    try {
      await deleteAgent.mutateAsync(deleteTarget.id);
      setDeleteTarget(null);
    } catch {
      setDeleteError("Could not delete agent. Please retry.");
    }
  }

  const editorMode: AgentFormMode | undefined = editor?.mode;
  const editorInitial = editor && editor.mode !== "create" ? editor.agent : undefined;

  return (
    <section aria-label="Agents workspace">
      <h2>Agents</h2>

      <button type="button" onClick={() => setEditor({ mode: "create" })}>
        New agent
      </button>

      {agentsQuery.isLoading && <p>Loading agents…</p>}
      {agentsQuery.isError && <p role="alert">Could not load agents. Please retry.</p>}

      {agentsQuery.data && (
        <AgentList
          agents={agentsQuery.data}
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
