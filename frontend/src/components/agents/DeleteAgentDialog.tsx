import type { Agent } from "../../lib/agents";
import { Alert, Button, Modal } from "../ui";

interface DeleteAgentDialogProps {
  agent: Agent;
  /** Flows that reference this agent. The F08 flows table does not exist yet,
   *  so this stays empty until F08 populates it (stable seam). */
  referencedFlows?: string[];
  onConfirm: () => void;
  onCancel: () => void;
  isDeleting?: boolean;
  error?: string | null;
}

/**
 * Confirmation before deleting an agent. When flows reference the agent it
 * warns which ones (the referenced-flows surface is wired for F08; until then
 * `referencedFlows` is empty and the warning is omitted).
 */
export function DeleteAgentDialog({
  agent,
  referencedFlows = [],
  onConfirm,
  onCancel,
  isDeleting = false,
  error = null,
}: DeleteAgentDialogProps) {
  return (
    <Modal
      open
      onClose={onCancel}
      title="Delete agent"
      size="sm"
      busy={isDeleting}
      footer={
        <>
          <Button variant="secondary" onClick={onCancel} disabled={isDeleting}>
            Cancel
          </Button>
          <Button
            variant="danger"
            onClick={onConfirm}
            disabled={isDeleting}
            loading={isDeleting}
            loadingLabel="Deleting…"
          >
            Delete
          </Button>
        </>
      }
    >
      <p style={{ margin: 0 }}>
        Delete <strong>{agent.name}</strong>? This cannot be undone.
      </p>

      {referencedFlows.length > 0 && (
        <div style={{ marginTop: "var(--space-3)" }}>
          <Alert variant="warning" role="alert" title="Referenced by flows">
            Removing this agent will flag its nodes as missing in:{" "}
            {referencedFlows.join(", ")}.
          </Alert>
        </div>
      )}

      {error && (
        <div style={{ marginTop: "var(--space-3)" }}>
          <Alert variant="danger" role="alert">
            {error}
          </Alert>
        </div>
      )}
    </Modal>
  );
}
