import type { Agent } from "../../lib/agents";

interface AgentListProps {
  agents: Agent[];
  onEdit: (agent: Agent) => void;
  onDuplicate: (agent: Agent) => void;
  onDelete: (agent: Agent) => void;
}

/**
 * The agent registry: one row per agent showing its name, provider, model, and
 * the recent-N / top-K caps, with edit, duplicate, and delete actions.
 */
export function AgentList({ agents, onEdit, onDuplicate, onDelete }: AgentListProps) {
  if (agents.length === 0) {
    return <p>No agents yet. Create one to get started.</p>;
  }

  return (
    <table>
      <thead>
        <tr>
          <th scope="col">Name</th>
          <th scope="col">Provider</th>
          <th scope="col">Model</th>
          <th scope="col">Recent-N</th>
          <th scope="col">Top-K</th>
          <th scope="col">Actions</th>
        </tr>
      </thead>
      <tbody>
        {agents.map((agent) => (
          <tr key={agent.id}>
            <td>{agent.name}</td>
            <td>{agent.provider}</td>
            <td>{agent.model}</td>
            <td>{agent.recent_n}</td>
            <td>{agent.top_k}</td>
            <td>
              <button type="button" onClick={() => onEdit(agent)}>
                Edit
              </button>
              <button type="button" onClick={() => onDuplicate(agent)}>
                Duplicate
              </button>
              <button type="button" onClick={() => onDelete(agent)}>
                Delete
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
