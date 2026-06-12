import { Button } from "../ui";
import styles from "./FlowToolbar.module.css";

interface FlowToolbarProps {
  /** Whether the graph has at least one node. */
  hasNodes: boolean;
  /** Whether a Root Agent has been assigned. */
  hasRoot: boolean;
  /** Count of nodes whose referenced agent was deleted from the registry. */
  missingAgentCount: number;
  /** True while a run is in flight (F10): the control reflects "Running…". */
  isRunning?: boolean;
  /**
   * The run action. F07 only wires the disabled-state UX; the actual execution
   * handler is supplied by F09/F10. When omitted, the control stays inert.
   */
  onRun?: () => void;
}

/**
 * Floating global toolbar (F07): the "Run Flow" control with its disabled
 * rules. Run is disabled until a Root Agent is assigned and at least one node
 * exists; a node referencing a deleted agent also blocks running. The actual
 * execution is a deferred seam for F09.
 */
export function FlowToolbar({
  hasNodes,
  hasRoot,
  missingAgentCount,
  isRunning = false,
  onRun,
}: FlowToolbarProps) {
  const message = !hasNodes
    ? "Add an agent to the canvas before running."
    : !hasRoot
      ? "Assign a Root Agent before running."
      : missingAgentCount > 0
        ? "Resolve missing agents before running."
        : null;

  const disabled = message !== null || isRunning;

  return (
    <div className={styles.toolbar} role="toolbar" aria-label="Flow actions">
      <Button
        variant="primary"
        disabled={disabled}
        loading={isRunning}
        loadingLabel="Running…"
        onClick={onRun}
        leadingIcon={
          <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" aria-hidden="true">
            <path d="M8 5v14l11-7z" />
          </svg>
        }
      >
        Run Flow
      </Button>
      {message && (
        <span className={styles.reason} role="status">
          {message}
        </span>
      )}
    </div>
  );
}
