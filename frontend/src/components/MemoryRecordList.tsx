import type { MemoryRecord } from "../lib/memory";

interface MemoryRecordListProps {
  records: MemoryRecord[];
  onEdit: (record: MemoryRecord) => void;
  onDelete: (record: MemoryRecord) => void;
}

/**
 * The stored memory records. Each listed record has been embedded already, so
 * it shows a "Ready" badge alongside its model and size, with edit/delete.
 */
export function MemoryRecordList({ records, onEdit, onDelete }: MemoryRecordListProps) {
  if (records.length === 0) {
    return <p>No memory records yet. Add one to embed and store it.</p>;
  }

  return (
    <ul aria-label="Memory records">
      {records.map((record) => (
        <li key={record.id}>
          <span>{record.text}</span>
          <span> · {record.embedding_model}</span>
          <span> · {record.char_count} chars</span>
          <span aria-label="status"> · Ready</span>
          <button type="button" onClick={() => onEdit(record)}>
            Edit
          </button>
          <button type="button" onClick={() => onDelete(record)}>
            Delete
          </button>
        </li>
      ))}
    </ul>
  );
}
