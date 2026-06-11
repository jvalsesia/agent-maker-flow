interface DeleteFlowDialogProps {
  flowName: string;
  onConfirm: () => void;
  onCancel: () => void;
  isDeleting?: boolean;
  error?: string | null;
}

/**
 * Confirmation before deleting a saved flow (F08). Deleting removes the saved
 * graph; it does not touch the agents it references.
 */
export function DeleteFlowDialog({
  flowName,
  onConfirm,
  onCancel,
  isDeleting = false,
  error = null,
}: DeleteFlowDialogProps) {
  return (
    <div role="dialog" aria-modal="true" aria-label={`Delete flow ${flowName}`}>
      <h3>Delete flow</h3>
      <p>
        Delete <strong>{flowName}</strong>? This cannot be undone.
      </p>

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
