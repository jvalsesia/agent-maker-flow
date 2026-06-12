import { useId, useState } from "react";

import { Alert, Button, Input, Modal } from "../ui";

/** Min/max trimmed length of a flow name — mirrors the backend rule (F08). */
export const NAME_MAX = 80;

/** Inline name validation mirroring the server (`FLOW_VALIDATION`). */
function validateName(name: string): string | null {
  const trimmed = name.trim();
  if (trimmed.length === 0) return "Name is required.";
  if ([...trimmed].length > NAME_MAX) return `Name must be ${NAME_MAX} characters or fewer.`;
  return null;
}

interface SaveFlowDialogProps {
  /** "save" names a new flow (Save As); "rename" renames an existing one. */
  mode: "save" | "rename";
  /** Prefilled name (the current flow's name in rename mode). */
  initialName?: string;
  onSubmit: (name: string) => void;
  onCancel: () => void;
  isSubmitting?: boolean;
  /** Server-side error (e.g. the `FLOW_NAME_TAKEN` duplicate-name message). */
  error?: string | null;
}

/**
 * Name-entry dialog reused for "Save as" and "Rename" (F08). It validates the
 * name inline (required, ≤80 chars) and disables submit until it is valid, and
 * surfaces the server's duplicate-name message when a save/rename is rejected.
 */
export function SaveFlowDialog({
  mode,
  initialName = "",
  onSubmit,
  onCancel,
  isSubmitting = false,
  error = null,
}: SaveFlowDialogProps) {
  const [name, setName] = useState(initialName);
  const formId = useId();
  const validationError = validateName(name);
  const count = [...name.trim()].length;
  const title = mode === "rename" ? "Rename flow" : "Save flow";
  const submitLabel = mode === "rename" ? "Rename" : "Save";

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    if (validationError) return;
    onSubmit(name.trim());
  };

  return (
    <Modal
      open
      onClose={onCancel}
      title={title}
      busy={isSubmitting}
      footer={
        <>
          <Button variant="secondary" onClick={onCancel} disabled={isSubmitting}>
            Cancel
          </Button>
          <Button
            type="submit"
            form={formId}
            variant="primary"
            disabled={isSubmitting || validationError !== null}
            loading={isSubmitting}
            loadingLabel={`${submitLabel}…`}
          >
            {submitLabel}
          </Button>
        </>
      }
    >
      <form id={formId} onSubmit={handleSubmit}>
        <Input
          label="Name"
          value={name}
          autoFocus
          onChange={(event) => setName(event.target.value)}
          error={validationError}
          counter={`${count} / ${NAME_MAX}`}
          counterOver={count > NAME_MAX}
        />
        {error && (
          <div style={{ marginTop: "var(--space-3)" }}>
            <Alert variant="danger" role="alert">
              {error}
            </Alert>
          </div>
        )}
      </form>
    </Modal>
  );
}
