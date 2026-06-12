# Implementation Plan: Conversational Monitor & Real-time Streaming

**Prerequisites:**
- Frontend: React + Vite + TypeScript, TanStack Query, `@xyflow/react`, Vitest + React Testing Library — all already configured.
- F09 complete and reachable: `POST /api/v1/runs`, `GET /api/v1/runs/{id}/events` (SSE), and `GET /api/v1/runs/{id}` (snapshot), with the ordered event contract (`run.started`, `node.started`, `node.partial`, `node.completed`, `node.failed`, `node.skipped`, `run.finished`).
- F01/F04 in place: `lib/apiClient.ts` (`apiPost`/`apiGet` + envelope + `ApiClientError`).
- F02 in place: `lib/authToken.ts` `sseUrlWithToken` (token query param for SSE) and the module-level token bridge; `hooks/useEventSource.ts` as the `EventSource`-lifecycle reference.
- F07/F08 in place: `lib/flowGraph.ts` (`FlowGraph`), `lib/useFlowGraph.ts`, `FlowToolbar` (with the unused `onRun?`/disabled-rules seam), `AgentNode` + `FlowNodeContextValue`, `FlowCanvas`, and `pages/FlowsPage.tsx` composition that owns the graph and the open-flow id.
- No backend changes and no new environment variables — F10 is frontend-only.

## Stage 1: Run Dispatch and Stream Reducer

**1. Run Client and Event Types** - Create `lib/runs.ts` with the TypeScript types for the seven F09 execution events and their payloads (mirroring the F09 wire contract), the `RunAccepted` result type, and `startRun(input)` built on `apiPost('/runs', …)` sending `prompt`, `graph`, and optional `flowId`. Add `lib/runs.test.ts` covering request shaping and rejection-code mapping per the spec's Testing Strategy.

**2. Run Stream Reducer** - Create `lib/runStream.ts` with `RunState`/`NodeRunState`, `initialRunState`, `parseRunEvent(name, data)`, and the pure `reduceRunEvent(state, event)` that folds each event type into per-node status/output, terminal status, and aggregated output, dropping any event whose `seq` is not newer than `lastSeq`. Add `lib/runStream.test.ts` covering every event transition, delta accumulation, the empty-output flag, and replay idempotency per the spec.

## Stage 2: SSE Run Hook

**3. useRunStream Hook** - Create `hooks/useRunStream.ts` that, given a `runId`, builds the authenticated events URL via `sseUrlWithToken`, opens an `EventSource`, registers a listener for each named F09 event, folds events through `reduceRunEvent`, and exposes the live `RunState` plus a connection status (open/connecting/closed). Follow the lifecycle pattern from `hooks/useEventSource.ts`. On a stream error where `run.finished` was not received, fetch `GET /runs/{id}` once via `apiGet` and fold its buffered `events[]` to recover the terminal state; surface "connecting" so the UI can show "Reconnecting…".

## Stage 3: Monitor UI and Page Composition

**4. Monitor Presentational Components** - Build `components/monitor/PromptBar.tsx` (prompt textarea + submit, blocked on empty/while running), `components/monitor/NodeBlock.tsx` (one agent block with a status light and its streamed/final/error output), and `components/monitor/ConversationTurns.tsx` (ordered user/assistant/system turns). Reference the spec's status → surface mapping for the light states.

**5. Conversation Monitor Panel** - Build `components/monitor/ConversationMonitor.tsx` composing the prompt bar, turn history, the per-node block list (derived from `RunState.nodes`), the "Reconnecting…" indicator, and the rejection/empty-output system lines. Add `components/monitor/ConversationMonitor.test.tsx` and `components/monitor/NodeBlock.test.tsx` per the spec.

**6. Canvas Status Badge** - Extend `components/flow/AgentNode.tsx` to read a per-node `status` from `FlowNodeContextValue` and render the idle/running/complete/error badge, and thread a `nodeStatuses` map through `components/flow/FlowCanvas.tsx` into the node context. Extend `components/flow/AgentNode.test.tsx` with the status-badge cases.

**7. FlowsPage Run Wiring** - Update `pages/FlowsPage.tsx` to own the run state (current `runId`, submitted prompt, session conversation turns, and the node-status map), wire `FlowToolbar.onRun` to `startRun` (rendering a system turn on a rejected start) and then drive `useRunStream`, render the `ConversationMonitor` as a right-split beside the canvas, append the assistant turn on `run.finished` (or the empty-output notice), and pass the derived `nodeStatuses` into the canvas context. Pass an `isRunning` flag into `FlowToolbar` to reflect the active run.

## Verification (each stage)

- Frontend stages: `npm test` + `npm run build`.
- One commit per stage, on `master`, pushed to `origin`, following the `feat(F10): stage N — …` convention used by F07–F09.
