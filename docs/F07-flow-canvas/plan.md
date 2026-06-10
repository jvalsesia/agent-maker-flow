# Implementation Plan: Flow Canvas

**Prerequisites:**
- Frontend: React 18 + Vite + TypeScript, TanStack Query, Clerk, react-router; the protected app shell with the existing `/flows` route and `FlowsPage` placeholder.
- F04 in place: the `useAgents` hook returning the caller's agent profiles (id, name, model).
- F02 in place: the `/flows` route already sits behind `RequireAuth`.
- New frontend dependency: `@xyflow/react` (v12).
- No backend work, no database migration, and no new endpoints are introduced by this feature.

## Stage 1: Graph Model & Validation Core

**1. Dependency** - Add `@xyflow/react` to the frontend manifest.

**2. Graph Model & Pure Helpers** - Add `lib/flowGraph.ts` with the `FlowGraph`/`FlowNodeData` types and the pure helpers from the spec's Internal Model Contract (cycle detection, connection validation, add/remove/duplicate/detach, set-root, can-run, missing-agent detection), plus `lib/flowGraph.test.ts` covering self-loops, direct/indirect cycles, duplicate edges, the single-root invariant, the node operations, the run gate, and missing-agent detection.

## Stage 2: Canvas Components

**3. Stateful Hook** - Add `lib/useFlowGraph.ts` wrapping React Flow's node/edge state with the domain invariants (validated connect, single root, node operations) and exposing the derived serializable `FlowGraph`.

**4. Agent Node** - Build `components/flow/AgentNode.tsx`: the custom node rendering the agent's name and model, input/output handles, the root badge, the "Agent missing" flag, and per-node actions (delete, duplicate, detach, set-root).

**5. Palette & Toolbar** - Build `components/flow/AgentPalette.tsx` (registry agents from `useAgents` as draggable items carrying the agent id) and `components/flow/FlowToolbar.tsx` (the floating "Run Flow" control with the disabled rules and inline messaging).

**6. Canvas** - Build `components/flow/FlowCanvas.tsx`: the `<ReactFlow>` surface with the custom node type, the drop target that instantiates a node from a dragged agent, and `onConnect`/`isValidConnection` wired to the validation helpers so invalid edges are rejected inline.

## Stage 3: Page Composition & Tests

**7. Flows Page** - Replace the `FlowsPage` placeholder to compose the palette, canvas, and toolbar, own the graph state via `useFlowGraph`, manage the create/connect/root/node-operation flows, and surface the inline validation and agent-missing messages.

**8. Component Tests & Test Env** - Add the `ResizeObserver` mock to the test setup, and add component tests for the toolbar (Run-Flow disabled rules), the palette (registry agents rendered as draggable items, mapping to the cross-feature criterion), and the agent node (label, root badge, agent-missing flag), per the spec's Testing Strategy.
