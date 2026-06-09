import type { Agent } from "../../lib/agents";

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
    <div role="dialog" aria-modal="true" aria-label={`Delete agent ${agent.name}`}>
      <h3>Delete agent</h3>
      <p>
        Delete <strong>{agent.name}</strong>? This cannot be undone.
      </p>

      {referencedFlows.length > 0 && (
        <div role="alert">
          <p>This agent is referenced by the following flows:</p>
          <ul>
            {referencedFlows.map((flow) => (
              <li key={flow}>{flow}</li>
            ))}
          </ul>
        </div>
      )}

      {error && <p role="alert">{error}</p>}

      <div>
        <button type="button" onClick={onConfirm} disabled={isDeleting}>
          Delete
        </button>
        <button type="button" onClick={onCancel} disabled={isDeleting}>
          Cancel
        </button>
      </div>
    </div>
  );
}
