import type { FlowSummary } from "../../lib/flows";
import { Button } from "../ui";
import styles from "./SavedFlowsList.module.css";

interface SavedFlowsListProps {
  flows: FlowSummary[];
  /** The flow currently open on the canvas, marked as the current row. */
  activeFlowId?: string | null;
  onOpen: (flow: FlowSummary) => void;
  onRename: (flow: FlowSummary) => void;
  onDelete: (flow: FlowSummary) => void;
}

/** Render the last-updated timestamp; falls back to the raw value if unparsable. */
function lastUpdated(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

/**
 * The caller's saved flows (F08): one row per flow with its name, a
 * last-updated indicator, and open / rename / delete actions. Action buttons
 * are labelled per flow so each is independently addressable; the active flow's
 * row is marked `aria-current`.
 */
export function SavedFlowsList({
  flows,
  activeFlowId = null,
  onOpen,
  onRename,
  onDelete,
}: SavedFlowsListProps) {
  if (flows.length === 0) {
    return <p className={styles.empty}>No saved flows yet. Save the canvas to create one.</p>;
  }

  return (
    <table className={styles.table} aria-label="Saved flows">
      <thead className={styles.srHead}>
        <tr>
          <th scope="col">Name</th>
          <th scope="col">Last updated</th>
          <th scope="col">Actions</th>
        </tr>
      </thead>
      <tbody>
        {flows.map((flow) => (
          <tr
            key={flow.id}
            className={styles.row}
            aria-current={flow.id === activeFlowId ? "true" : undefined}
          >
            <td className={styles.nameCell}>{flow.name}</td>
            <td className={styles.timeCell}>
              <time dateTime={flow.updated_at}>{lastUpdated(flow.updated_at)}</time>
            </td>
            <td className={styles.actionsCell}>
              <Button
                variant="ghost"
                size="sm"
                aria-label={`Open ${flow.name}`}
                onClick={() => onOpen(flow)}
              >
                Open
              </Button>
              <Button
                variant="ghost"
                size="sm"
                aria-label={`Rename ${flow.name}`}
                onClick={() => onRename(flow)}
              >
                Rename
              </Button>
              <Button
                variant="ghost"
                size="sm"
                aria-label={`Delete ${flow.name}`}
                onClick={() => onDelete(flow)}
              >
                Delete
              </Button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
