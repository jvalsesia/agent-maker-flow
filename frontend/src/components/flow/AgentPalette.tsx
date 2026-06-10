import { useAgents } from "../../lib/agents";

/** DataTransfer MIME type carrying the dragged agent's id onto the canvas. */
export const AGENT_DRAG_MIME = "application/x-agent-id";

/**
 * The agent palette (F07): the caller's F04 registry agents rendered as
 * draggable items. Each item carries its agent id via DataTransfer so the
 * canvas can instantiate a node referencing that agent on drop. This realizes
 * the cross-feature criterion that registry profiles appear as draggable nodes.
 */
export function AgentPalette() {
  const { data: agents, isLoading, isError } = useAgents();

  return (
    <aside aria-label="Agent palette" style={{ minWidth: 200 }}>
      <h3>Agents</h3>

      {isLoading && <p>Loading agents…</p>}
      {isError && <p role="alert">Could not load agents. Please retry.</p>}
      {agents && agents.length === 0 && (
        <p>No agents yet. Create one on the Agents dashboard.</p>
      )}

      {agents && agents.length > 0 && (
        <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
          {agents.map((agent) => (
            <li
              key={agent.id}
              draggable
              data-agent-id={agent.id}
              onDragStart={(event) => {
                event.dataTransfer.setData(AGENT_DRAG_MIME, agent.id);
                event.dataTransfer.effectAllowed = "move";
              }}
              style={{
                border: "1px solid #ccc",
                borderRadius: 4,
                padding: "6px 8px",
                marginBottom: 6,
                cursor: "grab",
              }}
            >
              <strong>{agent.name}</strong>
              <br />
              <small>{agent.model}</small>
            </li>
          ))}
        </ul>
      )}
    </aside>
  );
}
