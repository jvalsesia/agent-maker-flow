# Implementation Plan: Flow Persistence

**Prerequisites:**
- Backend: Rust + Axum 0.8, sqlx 0.8 (Postgres, uuid + chrono features), running PostgreSQL (`pgcrypto`/`pgvector` from `0001_init.sql`), Redis.
- F02 auth in place: `require_auth` middleware, `AuthUser` extractor, `ensure_owner`, `users` table (`0002_users.sql`); the `/flows` route already sits behind `RequireAuth`.
- F04 in place: the agents module/repo/service/routes pattern this feature mirrors exactly, plus the `useAgents` hook (used for the missing-agents banner) and the shared `apiClient` (GET/POST/PUT/DELETE).
- F07 in place: `lib/flowGraph.ts` (`FlowGraph` type + `missingAgentIds`), `lib/useFlowGraph.ts`, and the canvas/palette/toolbar components composed by `FlowsPage`.
- Environment: `DATABASE_URL`, `REDIS_URL`, gateway base URL configured as for F03/F04.
- New: migration `0005_flows.sql` (the project's first `jsonb` column); one new endpoint shape (`PATCH` for rename) beyond the agents template; `apiPatch` added to the frontend client.

## Stage 1: Data Model and Persistence

**1. Flows Migration** - Add `migrations/0005_flows.sql` creating the `flows` table owned by the user (`owner_id TEXT` FK to `users(id)` ON DELETE CASCADE), with the `graph jsonb` column, the case-insensitive per-user unique index `ux_flows_owner_name` on `(owner_id, lower(name))`, and the `ix_flows_owner` lookup index. Follow the spec's Data Model section for exact columns and indexes.

**2. Flows Module and Repository** - Create the `flows` backend module with `model.rs` (`Flow`, `FlowSummary`, the `FlowGraph` jsonb payload bound via sqlx `Json<FlowGraph>`, `CreateFlowInput`, `RenameInput`, and the `NAME_MIN`/`NAME_MAX`/`GRAPH_MAX_BYTES` constants) and a sqlx `repo.rs` with the `owner_id`-scoped queries: `insert`, `list_summaries_by_owner` (ordered by `updated_at` desc, no graph), `get`, `update`, `rename`, `delete`, and `name_exists` (excluding-self variant for update/rename). Add `mod.rs` re-exporting model/service like `agents/mod.rs`, and declare `pub mod flows;` in `lib.rs`.

## Stage 2: Service, Errors, and Endpoints

**3. Error Codes** - Add the stable codes `FLOW_VALIDATION` (422, on `AppError::Validation`) and `FLOW_NAME_TAKEN` (409, on `AppError::Conflict`), reusing the existing variants and HTTP-status mapping; `NotFound` (404) is reused as-is for missing/cross-user rows.

**4. Flows Service** - Implement `service.rs`: validate the name (trimmed 1–80 chars) and the graph structure (well-formed `nodes`/`edges` arrays; `rootNodeId` null or a present node id) under the `GRAPH_MAX_BYTES` serialized-size cap; enforce per-user case-insensitive uniqueness (excluding self on update/rename) mapping a clash to `FLOW_NAME_TAKEN`; apply the owner-scope guard so missing/other-owner rows surface as `NotFound`. DAG/cycle enforcement stays in F07 and is not re-implemented here. Include `#[cfg(test)]` unit tests for the validation rules (valid input, empty/whitespace name, name at/over 80, malformed graph, oversized graph) per the spec.

**5. Flows Routes** - Add `routes/flows.rs` with the six protected handlers (`create` → 201, `list`, `get`, `update`, `rename`, `delete`) that pull the authenticated user, delegate to the service, and render the success envelope. Mount `/flows` and `/flows/{id}` (including the `patch` for rename) on the protected router in `routes/mod.rs`.

**6. Backend Integration Tests** - Add `tests/flows_test.rs` using the existing auth test harness, covering the spec's cases against the live DB: `create_then_get_round_trips_graph`, `list_returns_summaries_without_graph`, `duplicate_name_is_rejected` (incl. case-variant), `update_saves_graph_and_refreshes_updated_at` (same-name re-save allowed), `rename_enforces_uniqueness`, `delete_removes_flow`, and `cross_user_isolation` (B's GET/PUT/PATCH/DELETE of A's flow → 404; B's list excludes A's).

## Stage 3: Frontend Data Layer

**7. API Client PATCH Verb** - Add an `apiPatch` helper to `lib/apiClient.ts` built on the existing request/envelope handling (the client currently has GET/POST/PUT/DELETE only), for the rename call.

**8. Flows Hooks** - Add `lib/flows.ts` with the `Flow`/`FlowSummary` types and the TanStack Query hooks `useFlows`, `useFlow`, `useCreateFlow`, `useUpdateFlow`, `useRenameFlow`, and `useDeleteFlow`, each invalidating the flows query on success. Add `lib/flows.test.ts` covering the hooks and request shaping per the spec.

## Stage 4: Flow Persistence UI

**9. Canvas Load Capability** - Extend `lib/useFlowGraph.ts` with a `load(graph)` method that replaces nodes/edges/root in place (no remount), so an opened flow restores the canvas exactly as saved while keeping React Flow state stable.

**10. Dialogs and List** - Build `components/flow/SaveFlowDialog.tsx` (mode-driven name entry reused for "save as" and "rename", with inline 1–80 validation and the duplicate-name message mapped from `FLOW_NAME_TAKEN`), `components/flow/DeleteFlowDialog.tsx` (delete confirmation), `components/flow/SavedFlowsList.tsx` (the user's flows with open/rename/delete actions and a last-updated indicator), and `components/flow/MissingAgentsBanner.tsx` (lists missing agents on open, reusing F07 `missingAgentIds` joined against `useAgents`).

**11. Flows Page Composition and Tests** - Update `pages/FlowsPage.tsx` to compose the saved-flows list, dialogs, and banner; own the open/dirty/save state with a saved snapshot driving the unsaved-changes guard and the toolbar Save/Save As; call `useFlowGraph.load` on open. Add the component tests for `SaveFlowDialog` (name validation + duplicate message, rename mode), `SavedFlowsList` (render + open/rename/delete actions), and `MissingAgentsBanner` (missing-agents listing), per the spec's Testing Strategy.

## Verification (each stage)

- Backend stages: `cargo build` + `cargo clippy --all-targets` + `cargo test` (live DB + Redis + mock gateway are reachable, so the DB-backed `flows_test.rs` cases execute, not skip). Do **not** run `cargo fmt` — match the existing hand-formatted style.
- Frontend stages: `npm test` + `npm run build`.
- One commit per stage, on `master`, pushed to `origin`, following the `feat(F08): stage N — …` convention.
