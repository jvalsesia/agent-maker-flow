import type { MemoryRecord } from "../lib/memory";
import { Badge, Button, StatusDot } from "./ui";
import styles from "./MemoryRecordList.module.css";

interface MemoryRecordListProps {
  records: MemoryRecord[];
  /** Agent id → name, used to label a record scoped to a single agent (F06). */
  agentNames?: Record<string, string>;
  onEdit: (record: MemoryRecord) => void;
  onDelete: (record: MemoryRecord) => void;
}

/**
 * The stored memory records. Each listed record has been embedded already, so
 * it shows a "Ready" status alongside its model and size, with edit/delete.
 */
export function MemoryRecordList({
  records,
  agentNames = {},
  onEdit,
  onDelete,
}: MemoryRecordListProps) {
  if (records.length === 0) {
    return <p className={styles.empty}>No memory records yet. Add one to embed and store it.</p>;
  }

  return (
    <ul className={styles.list} aria-label="Memory records">
      {records.map((record) => (
        <li key={record.id} className={styles.item}>
          <div className={styles.body}>
            <span className={styles.text}>{record.text}</span>
            <div className={styles.meta}>
              <span className={styles.status} aria-label="status">
                <StatusDot status="complete" label="" />
                Ready
              </span>
              <span aria-hidden="true">·</span>
              <Badge variant="neutral" className={styles.model}>
                {record.embedding_model}
              </Badge>
              <span aria-hidden="true">·</span>
              <span>{record.char_count} chars</span>
              {record.agent_id && (
                <>
                  <span aria-hidden="true">·</span>
                  <Badge variant="accent">
                    {agentNames[record.agent_id]
                      ? `Private to ${agentNames[record.agent_id]}`
                      : "Agent-scoped"}
                  </Badge>
                </>
              )}
            </div>
          </div>
          <div className={styles.actions}>
            <Button variant="ghost" size="sm" aria-label={`Edit record`} onClick={() => onEdit(record)}>
              Edit
            </Button>
            <Button
              variant="ghost"
              size="sm"
              aria-label={`Delete record`}
              onClick={() => onDelete(record)}
            >
              Delete
            </Button>
          </div>
        </li>
      ))}
    </ul>
  );
}
