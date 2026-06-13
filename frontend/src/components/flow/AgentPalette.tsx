import { useMemo, useState } from "react";

import { useAgents } from "../../lib/agents";
import { Alert, Badge, Button, Input, SkeletonRows } from "../ui";
import styles from "./AgentPalette.module.css";

/** DataTransfer MIME type carrying the dragged agent's id onto the canvas. */
export const AGENT_DRAG_MIME = "application/x-agent-id";

/** Above this many agents, show a search filter to keep the palette scannable. */
const SEARCH_THRESHOLD = 8;

interface AgentPaletteProps {
  /** Keyboard fallback for the mouse-only drag-and-drop: adds the agent to the
   *  canvas at a default position (spec §7 canvas a11y). */
  onAddToCanvas?: (agentId: string) => void;
}

/**
 * The agent palette (F07): the caller's F04 registry agents rendered as
 * draggable items. Each item carries its agent id via DataTransfer so the
 * canvas can instantiate a node referencing that agent on drop. This realizes
 * the cross-feature criterion that registry profiles appear as draggable nodes.
 */
export function AgentPalette({ onAddToCanvas }: AgentPaletteProps = {}) {
  const { data: agents, isLoading, isError } = useAgents();
  const [query, setQuery] = useState("");

  const filtered = useMemo(() => {
    const all = agents ?? [];
    const q = query.trim().toLowerCase();
    if (!q) return all;
    return all.filter(
      (a) => a.name.toLowerCase().includes(q) || a.model.toLowerCase().includes(q),
    );
  }, [agents, query]);

  return (
    <div className={styles.palette} aria-label="Agent palette">
      <div className={styles.header}>
        <span className={styles.title}>Agents</span>
        {agents && agents.length > 0 && <Badge variant="neutral">{agents.length}</Badge>}
      </div>

      {agents && agents.length > SEARCH_THRESHOLD && (
        <Input
          type="search"
          aria-label="Filter agents"
          placeholder="Filter agents…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      )}

      {isLoading && <SkeletonRows count={4} />}
      {isError && (
        <Alert variant="danger" role="alert">
          Could not load agents. Please retry.
        </Alert>
      )}
      {agents && agents.length === 0 && (
        <p className={styles.empty}>No agents yet. Create one on the Agents dashboard.</p>
      )}

      {filtered.length > 0 && (
        <ul className={styles.list}>
          {filtered.map((agent) => (
            <li
              key={agent.id}
              className={styles.item}
              draggable
              data-agent-id={agent.id}
              title={`Drag ${agent.name} onto the canvas`}
              onDragStart={(event) => {
                event.dataTransfer.setData(AGENT_DRAG_MIME, agent.id);
                event.dataTransfer.effectAllowed = "move";
              }}
            >
              <div className={styles.itemHead}>
                <span className={styles.name}>{agent.name}</span>
                <Badge variant="neutral" dot>
                  {agent.provider}
                </Badge>
              </div>
              <div className={styles.itemFoot}>
                <span className={styles.model}>{agent.model}</span>
                {onAddToCanvas && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className={styles.add}
                    aria-label={`Add ${agent.name} to canvas`}
                    title="Add to canvas"
                    onClick={() => onAddToCanvas(agent.id)}
                  >
                    + Add
                  </Button>
                )}
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
