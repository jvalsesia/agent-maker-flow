import { useState } from "react";

import { ApiClientError } from "../lib/apiClient";
import {
  MEMORY_TEXT_MAX,
  useCreateMemoryRecord,
  useUpdateMemoryRecord,
  type MemoryRecord,
} from "../lib/memory";

interface MemoryRecordFormProps {
  /** Record to edit; omitted for create mode. */
  record?: MemoryRecord;
  onSaved?: () => void;
  onCancel?: () => void;
}

/**
 * Add/edit a memory record. Shows a live character counter against the 8,000
 * limit, blocks submit when empty or over the limit, and reflects the
 * embed-on-save lifecycle: "Embedding…" while the request is in flight, a
 * stored confirmation on success, and any embed error inline.
 */
export function MemoryRecordForm({ record, onSaved, onCancel }: MemoryRecordFormProps) {
  const isEditing = record != null;
  const [text, setText] = useState(record?.text ?? "");

  const createRecord = useCreateMemoryRecord();
  const updateRecord = useUpdateMemoryRecord();
  const active = isEditing ? updateRecord : createRecord;

  const length = text.length;
  const overLimit = length > MEMORY_TEXT_MAX;
  const empty = text.trim().length === 0;
  const canSubmit = !empty && !overLimit && !active.isPending;

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!canSubmit) return;
    try {
      if (isEditing) {
        await updateRecord.mutateAsync({ id: record.id, text });
      } else {
        await createRecord.mutateAsync(text);
        setText("");
      }
      onSaved?.();
    } catch {
      // Error surfaced below via the mutation's error state.
    }
  }

  const errorMessage =
    active.error instanceof ApiClientError
      ? active.error.message
      : active.error
        ? "Could not save the record. Please retry."
        : null;

  return (
    <form onSubmit={handleSubmit} aria-label={isEditing ? "Edit memory record" : "Add memory record"}>
      <label htmlFor="memory-text">Memory text</label>
      <br />
      <textarea
        id="memory-text"
        value={text}
        onChange={(e) => setText(e.target.value)}
        aria-invalid={overLimit ? true : undefined}
      />
      <p aria-label="character count">
        {length} / {MEMORY_TEXT_MAX}
      </p>
      {overLimit && <p role="alert">Memory record must be 8000 characters or fewer.</p>}

      {active.isPending && <p role="status">Embedding…</p>}
      {active.isSuccess && !active.isPending && <p role="status">Stored.</p>}
      {errorMessage && <p role="alert">{errorMessage}</p>}

      <div>
        <button type="submit" disabled={!canSubmit}>
          {active.isPending ? "Embedding…" : isEditing ? "Save changes" : "Add record"}
        </button>
        {onCancel && (
          <button type="button" onClick={onCancel} disabled={active.isPending}>
            Cancel
          </button>
        )}
      </div>
    </form>
  );
}
