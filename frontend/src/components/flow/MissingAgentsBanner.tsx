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
    <div role="alert" aria-label="Missing agents">
      <p>
        {missingAgentIds.length} node(s) reference an agent that no longer exists. Remove or
        repoint them before running.
      </p>
      <ul>
        {missingAgentIds.map((id) => (
          <li key={id}>{id}</li>
        ))}
      </ul>
    </div>
  );
}
