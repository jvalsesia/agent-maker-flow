import { useState } from "react";

import { useAgents } from "../lib/agents";
import { ApiClientError } from "../lib/apiClient";
import {
  MEMORY_TEXT_MAX,
  useCreateMemoryRecord,
  useUpdateMemoryRecord,
  type MemoryRecord,
} from "../lib/memory";
import { Alert, Button, Select, Spinner, Textarea } from "./ui";
import styles from "./MemoryRecordForm.module.css";

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
  // "" means a global record; any other value scopes it to that agent (F06).
  const [agentId, setAgentId] = useState(record?.agent_id ?? "");

  const agents = useAgents();
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
    const scopedAgentId = agentId === "" ? null : agentId;
    try {
      if (isEditing) {
        await updateRecord.mutateAsync({ id: record.id, text, agentId: scopedAgentId });
      } else {
        await createRecord.mutateAsync({ text, agentId: scopedAgentId });
        setText("");
        setAgentId("");
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
    <form
      className={styles.form}
      onSubmit={handleSubmit}
      aria-label={isEditing ? "Edit memory record" : "Add memory record"}
    >
      <Textarea
        label="Memory text"
        mono
        rows={4}
        value={text}
        onChange={(e) => setText(e.target.value)}
        error={overLimit ? "Memory record must be 8000 characters or fewer." : undefined}
      />
      <div className={styles.counterRow}>
        <p
          aria-label="character count"
          className={[styles.counter, overLimit && styles.counterOver].filter(Boolean).join(" ")}
        >
          {length} / {MEMORY_TEXT_MAX}
        </p>
      </div>

      <Select
        label="Scope to agent"
        hint="Records scoped to an agent are only retrieved by that agent (when its memory scope is “own”)."
        value={agentId}
        onChange={(e) => setAgentId(e.target.value)}
        disabled={active.isPending}
      >
        <option value="">All agents (global)</option>
        {(agents.data ?? []).map((agent) => (
          <option key={agent.id} value={agent.id}>
            {agent.name}
          </option>
        ))}
      </Select>

      {active.isPending && (
        <p className={styles.status} role="status">
          <Spinner size="sm" label="" />
          Embedding…
        </p>
      )}
      {active.isSuccess && !active.isPending && (
        <p className={styles.status} role="status">
          Stored.
        </p>
      )}
      {errorMessage && (
        <Alert variant="danger" role="alert">
          {errorMessage}
        </Alert>
      )}

      <div className={styles.actions}>
        <Button
          type="submit"
          variant="primary"
          disabled={!canSubmit}
          loading={active.isPending}
          loadingLabel="Embedding…"
        >
          {isEditing ? "Save changes" : "Add record"}
        </Button>
        {onCancel && (
          <Button variant="secondary" onClick={onCancel} disabled={active.isPending}>
            Cancel
          </Button>
        )}
      </div>
    </form>
  );
}
