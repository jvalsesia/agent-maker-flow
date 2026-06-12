import styles from "./MissingAgentsBanner.module.css";

interface MissingAgentsBannerProps {
  /** Agent ids referenced by nodes but absent from the registry (F07
   *  `missingAgentIds`). Empty means every node resolves and nothing renders. */
  missingAgentIds: string[];
}

/**
 * Banner shown when an opened flow references agents that were deleted from the
 * registry (F08). It flags how many nodes are affected and lists the missing
 * agent ids so the user can repoint or remove them; running stays blocked while
 * any remain. Renders nothing when there are no missing agents.
 */
export function MissingAgentsBanner({ missingAgentIds }: MissingAgentsBannerProps) {
  if (missingAgentIds.length === 0) return null;

  return (
    <div className={styles.banner} role="alert" aria-label="Missing agents">
      <p className={styles.message}>
        <svg
          className={styles.icon}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          aria-hidden="true"
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v4m0 3h.01M12 3l9 16H3l9-16z" />
        </svg>
        {missingAgentIds.length} node(s) reference an agent that no longer exists. Remove or repoint
        them before running.
      </p>
      <ul className={styles.ids}>
        {missingAgentIds.map((id) => (
          <li key={id} className={styles.id}>
            {id}
          </li>
        ))}
      </ul>
    </div>
  );
}
