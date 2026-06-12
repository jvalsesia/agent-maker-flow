import type { Agent } from "../../lib/agents";
import { Badge, Button, Card, EmptyState } from "../ui";
import styles from "./AgentList.module.css";

interface AgentListProps {
  agents: Agent[];
  onEdit: (agent: Agent) => void;
  onDuplicate: (agent: Agent) => void;
  onDelete: (agent: Agent) => void;
  /** Optional create action for the empty state. */
  onCreate?: () => void;
}

const EditIcon = (
  <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" d="M4 20h4l10-10-4-4L4 16v4zM13.5 6.5l4 4" />
  </svg>
);

const DuplicateIcon = (
  <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
    <rect x="9" y="9" width="11" height="11" rx="2" />
    <path d="M5 15V5a2 2 0 012-2h10" />
  </svg>
);

const DeleteIcon = (
  <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
    <path strokeLinecap="round" d="M4 7h16M9 7V5a1 1 0 011-1h4a1 1 0 011 1v2m-8 0l1 13h8l1-13" />
  </svg>
);

/**
 * The agent registry as a Card-wrapped table: one row per agent showing its
 * name, provider tag, model, and the recent-N / top-K caps, with edit,
 * duplicate, and delete actions revealed on row hover/focus (always reachable
 * via keyboard).
 */
export function AgentList({ agents, onEdit, onDuplicate, onDelete, onCreate }: AgentListProps) {
  if (agents.length === 0) {
    return (
      <Card bodyPadded={false}>
        <EmptyState
          icon={
            <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
              <circle cx="12" cy="8" r="3.5" />
              <path strokeLinecap="round" d="M5 20a7 7 0 0114 0" />
            </svg>
          }
          title="No agents yet"
          description="Define a reusable LLM agent to drop onto your flows."
          action={
            onCreate && (
              <Button variant="primary" onClick={onCreate}>
                Create your first agent
              </Button>
            )
          }
        />
      </Card>
    );
  }

  return (
    <Card bodyPadded={false}>
      <table className={styles.table}>
        <thead className={styles.head}>
          <tr>
            <th scope="col">Name</th>
            <th scope="col">Provider</th>
            <th scope="col">Model</th>
            <th scope="col" className={styles.numHead}>
              Recent-N
            </th>
            <th scope="col" className={styles.numHead}>
              Top-K
            </th>
            <th scope="col" className={styles.numHead}>
              Actions
            </th>
          </tr>
        </thead>
        <tbody>
          {agents.map((agent) => (
            <tr key={agent.id} className={styles.row}>
              <td className={`${styles.cell} ${styles.name}`}>{agent.name}</td>
              <td className={styles.cell}>
                <Badge variant="neutral" dot>
                  {agent.provider}
                </Badge>
              </td>
              <td className={`${styles.cell} ${styles.model}`}>{agent.model}</td>
              <td className={`${styles.cell} ${styles.num}`}>{agent.recent_n}</td>
              <td className={`${styles.cell} ${styles.num}`}>{agent.top_k}</td>
              <td className={styles.actionsCell}>
                <span className={styles.actions}>
                  <Button
                    variant="ghost"
                    size="sm"
                    iconOnly
                    aria-label={`Edit ${agent.name}`}
                    title="Edit"
                    onClick={() => onEdit(agent)}
                  >
                    {EditIcon}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    iconOnly
                    aria-label={`Duplicate ${agent.name}`}
                    title="Duplicate"
                    onClick={() => onDuplicate(agent)}
                  >
                    {DuplicateIcon}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    iconOnly
                    aria-label={`Delete ${agent.name}`}
                    title="Delete"
                    onClick={() => onDelete(agent)}
                  >
                    {DeleteIcon}
                  </Button>
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </Card>
  );
}
