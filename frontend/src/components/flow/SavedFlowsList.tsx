import type { FlowSummary } from "../../lib/flows";

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
 * are labelled per flow so each is independently addressable.
 */
export function SavedFlowsList({
  flows,
  activeFlowId = null,
  onOpen,
  onRename,
  onDelete,
}: SavedFlowsListProps) {
  if (flows.length === 0) {
    return <p>No saved flows yet. Save the canvas to create one.</p>;
  }

  return (
    <table aria-label="Saved flows">
      <thead>
        <tr>
          <th scope="col">Name</th>
          <th scope="col">Last updated</th>
          <th scope="col">Actions</th>
        </tr>
      </thead>
      <tbody>
        {flows.map((flow) => (
          <tr key={flow.id} aria-current={flow.id === activeFlowId ? "true" : undefined}>
            <td>{flow.name}</td>
            <td>
              <time dateTime={flow.updated_at}>{lastUpdated(flow.updated_at)}</time>
            </td>
            <td>
              <button type="button" aria-label={`Open ${flow.name}`} onClick={() => onOpen(flow)}>
                Open
              </button>
              <button
                type="button"
                aria-label={`Rename ${flow.name}`}
                onClick={() => onRename(flow)}
              >
                Rename
              </button>
              <button
                type="button"
                aria-label={`Delete ${flow.name}`}
                onClick={() => onDelete(flow)}
              >
                Delete
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
