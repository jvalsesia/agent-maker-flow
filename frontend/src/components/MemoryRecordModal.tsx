import { useId, useState } from "react";

import { useAgents } from "../lib/agents";
import { ApiClientError } from "../lib/apiClient";
import {
  MEMORY_TEXT_MAX,
  useCreateMemoryRecord,
  useUpdateMemoryRecord,
  type MemoryRecord,
} from "../lib/memory";
import { Alert, Badge, Button, Modal, Select, Spinner, StatusDot, Textarea } from "./ui";
import styles from "./MemoryRecordModal.module.css";

export type MemoryRecordModalMode = "create" | "edit" | "view";

interface MemoryRecordModalProps {
  mode: MemoryRecordModalMode;
  /** Record to view or edit; omitted for create mode. */
  record?: MemoryRecord;
  /** Agent id → name, used to label an agent-scoped record (F06). */
  agentNames?: Record<string, string>;
  onClose: () => void;
  onSaved?: () => void;
  /** Switch a view modal into edit mode (the “Edit” footer action). */
  onEdit?: () => void;
}

function formatTimestamp(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

/**
 * Add / edit / view a memory record in a modal (F05/F06). Create and edit share
 * the controlled form — a live character counter against the 8,000 limit, a
 * blocked submit when empty or over the limit, and the embed-on-save lifecycle
 * ("Embedding…", stored confirmation, inline error). View mode renders the
 * stored text and metadata read-only, with an "Edit" action to switch modes.
 */
export function MemoryRecordModal({
  mode,
  record,
  agentNames = {},
  onClose,
  onSaved,
  onEdit,
}: MemoryRecordModalProps) {
  const isView = mode === "view";
  const isEditing = mode === "edit";
  const [text, setText] = useState(record?.text ?? "");
  // "" means a global record; any other value scopes it to that agent (F06).
  const [agentId, setAgentId] = useState(record?.agent_id ?? "");

  const agents = useAgents();
  const createRecord = useCreateMemoryRecord();
  const updateRecord = useUpdateMemoryRecord();
  const active = isEditing ? updateRecord : createRecord;
  const formId = useId();

  const length = text.length;
  const overLimit = length > MEMORY_TEXT_MAX;
  const empty = text.trim().length === 0;
  const canSubmit = !empty && !overLimit && !active.isPending;

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!canSubmit) return;
    const scopedAgentId = agentId === "" ? null : agentId;
    try {
      if (isEditing && record) {
        await updateRecord.mutateAsync({ id: record.id, text, agentId: scopedAgentId });
      } else {
        await createRecord.mutateAsync({ text, agentId: scopedAgentId });
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

  const title =
    mode === "edit" ? "Edit memory record" : mode === "view" ? "Memory record" : "Add memory record";

  // --- View mode: read-only text + metadata ---------------------------------
  if (isView && record) {
    const scopeLabel = record.agent_id
      ? agentNames[record.agent_id]
        ? `Private to ${agentNames[record.agent_id]}`
        : "Agent-scoped"
      : "All agents (global)";

    return (
      <Modal
        open
        onClose={onClose}
        title={title}
        size="md"
        footer={
          <>
            <Button variant="secondary" onClick={onClose}>
              Close
            </Button>
            {onEdit && (
              <Button variant="primary" onClick={onEdit}>
                Edit
              </Button>
            )}
          </>
        }
      >
        <div className={styles.view} aria-label="Memory record details">
          <pre className={styles.text}>{record.text}</pre>

          <dl className={styles.meta}>
            <div className={styles.metaRow}>
              <dt>Status</dt>
              <dd className={styles.status}>
                <StatusDot status="complete" label="" />
                Ready
              </dd>
            </div>
            <div className={styles.metaRow}>
              <dt>Embedding model</dt>
              <dd>
                <Badge variant="neutral">{record.embedding_model}</Badge>
              </dd>
            </div>
            <div className={styles.metaRow}>
              <dt>Size</dt>
              <dd>{record.char_count} chars</dd>
            </div>
            <div className={styles.metaRow}>
              <dt>Scope</dt>
              <dd>
                <Badge variant={record.agent_id ? "accent" : "neutral"}>{scopeLabel}</Badge>
              </dd>
            </div>
            <div className={styles.metaRow}>
              <dt>Created</dt>
              <dd>{formatTimestamp(record.created_at)}</dd>
            </div>
            <div className={styles.metaRow}>
              <dt>Updated</dt>
              <dd>{formatTimestamp(record.updated_at)}</dd>
            </div>
          </dl>
        </div>
      </Modal>
    );
  }

  // --- Create / edit mode: the controlled form ------------------------------
  return (
    <Modal
      open
      onClose={onClose}
      title={title}
      size="md"
      busy={active.isPending}
      footer={
        <>
          <Button variant="secondary" onClick={onClose} disabled={active.isPending}>
            Cancel
          </Button>
          <Button
            type="submit"
            form={formId}
            variant="primary"
            disabled={!canSubmit}
            loading={active.isPending}
            loadingLabel="Embedding…"
          >
            {isEditing ? "Save changes" : "Add record"}
          </Button>
        </>
      }
    >
      <form id={formId} className={styles.form} onSubmit={handleSubmit} aria-label={title}>
        <Textarea
          label="Memory text"
          mono
          rows={6}
          value={text}
          onChange={(e) => setText(e.target.value)}
          error={overLimit ? "Memory record must be 8000 characters or fewer." : undefined}
          counter={
            <span aria-label="character count">
              {length} / {MEMORY_TEXT_MAX}
            </span>
          }
          counterOver={overLimit}
        />

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
        {errorMessage && (
          <Alert variant="danger" role="alert">
            {errorMessage}
          </Alert>
        )}
      </form>
    </Modal>
  );
}
