import { Alert, Button, Modal } from "../ui";

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
    <Modal
      open
      onClose={onCancel}
      title="Delete flow"
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
        Delete <strong>{flowName}</strong>? This cannot be undone.
      </p>
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
