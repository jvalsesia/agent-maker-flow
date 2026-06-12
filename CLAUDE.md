# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A node-based multi-agent flow orchestrator. Users define reusable LLM **agents**, wire them into directed **flows** on a React Flow canvas, and execute the graph against a chat prompt. The Rust backend translates the canvas into a DAG, runs each agent through a LiteLLM gateway, forwards each node's output to its downstream nodes, and streams results back over SSE. See `project-intent.md` and `PRD.md` for the product vision.

## Repository layout

Two independently-tooled top-level apps (not a Cargo/npm workspace):

- `backend/` — Rust Axum (v0.8.9) service, REST under `/api/v1` + SSE.
- `frontend/` — React 18 + Vite + TypeScript SPA.
- `litellm/config.yaml` — model catalog for the LiteLLM proxy (the single gateway for every provider completion/embedding call). Add models here, not in the backend.
- `docs/F01..F09/` — per-feature `spec.md` + `plan.md` (see Feature workflow).

## Common commands

Start infra first — most backend work needs Postgres + Redis (+ LiteLLM):

```bash
docker compose up -d            # postgres (pgvector, :5432), redis (:6379), litellm (:4000)
```

Backend (`cd backend`):

```bash
cargo run                       # boots: connect+migrate Postgres, PING Redis, warm JWKS, serve :8080
cargo test                      # integration tests (soft-skip when DB/Redis unreachable — see below)
cargo test --test flows_test    # one test file
cargo test --test flows_test create_flow   # one test by name
cargo clippy --all-targets
cargo fmt
```

Frontend (`cd frontend`):

```bash
npm install
npm run dev                     # Vite :5173, proxies /api -> localhost:8080
npm test                        # vitest run (jsdom)
npm test -- AgentForm           # one test file by name match
npm run build                   # tsc --noEmit && vite build
npm run typecheck
```

Copy `backend/.env.example` → `backend/.env` and `frontend/.env.example` → `frontend/.env` before running. `CLERK_ISSUER` and `CLERK_AUTHORIZED_PARTIES` are required by the backend; `VITE_CLERK_PUBLISHABLE_KEY` by the frontend.

## Backend architecture

**Boot is fail-fast** (`main.rs`): it connects to Postgres, runs migrations, verifies the `pgvector` extension, PINGs Redis, and warms the Clerk JWKS cache *before* binding the listener. Misconfiguration surfaces at startup, not as runtime 500s.

**Shared state** is `AppState` (`state.rs`) — `db: PgPool`, `redis: deadpool-redis Pool`, `config`, `auth: Arc<AuthState>`, `gateway: Arc<GatewayClient>`. Cheap to clone; injected into every handler.

**Feature modules follow a consistent shape.** Each domain (`agents/`, `flows/`, `memory/`, `gateway/`, `auth/`) is a module folder, and most expose:
- `model.rs` — domain types + input DTOs (serde).
- `repo.rs` — owner-scoped `sqlx` queries (hand-written SQL; no ORM).
- `service.rs` — validation, uniqueness, and gateway orchestration.
- `mod.rs` — re-exports the public surface.

HTTP handlers live separately in `routes/*.rs` and call into the service layer. `lib.rs` exposes everything so both the binary and the `tests/` integration suite can build the app.

**Routing** (`routes/mod.rs`): `/health` is public; everything else (`/me`, `/providers`, `/agents`, `/flows`, `/memory`, `/settings/*`, `/sse/heartbeat`) sits behind the `require_auth` Clerk-JWT middleware. Ownership is enforced in the repo layer — queries are scoped to the authenticated user, and a record owned by another user returns `NotFound` (never reveals existence).

**Response envelope is platform-wide and load-bearing** — the frontend `apiClient` parses exactly these shapes:
- success: `{ "status": "success", "data": <T> }`
- error: `{ "status": "error", "error": { "code", "message" } }`

All errors flow through `AppError` in `error.rs` (`thiserror` + `IntoResponse`), which owns the status-code and stable-`code` mapping (e.g. `AGENT_VALIDATION`, `AGENT_NAME_TAKEN`). Add new failure modes as `AppError` variants rather than building ad-hoc responses.

**LLM access** goes through `gateway/` (`GatewayClient`) to the LiteLLM proxy — never call providers directly. The gateway also handles completion caching (sha2 cache keys) and usage counters in Redis. Provider API keys are consumed by the LiteLLM *container*, not the backend.

**Migrations** are `backend/migrations/NNNN_*.sql`, applied automatically at boot via `sqlx::migrate`. Add a new numbered file; do not edit applied ones.

### Tests require live infra, and soft-skip without it

Integration tests in `backend/tests/` spin up the real Axum app against `DATABASE_URL`/`REDIS_URL`. When those env vars are absent/unreachable they print `SKIP:` and return green, so `cargo test` passes locally without Docker but only *exercises* anything when the stack is up. Auth-protected tests mint Clerk-style JWTs with an embedded test RSA keypair and a mock LiteLLM proxy router; each test uses a freshly-nonced owner id so the suite is rerunnable against a persistent DB.

## Frontend architecture

- **Routing**: React Router v6 (`routes/router.tsx`) inside a persistent `AppShell` + `NavBar`. Workspaces: `/agents`, `/flows`, `/settings`.
- **Auth**: Clerk (`@clerk/clerk-react`). `RequireAuth` guards routes; `AuthTokenBridge` + `authToken.ts` stash the session token so the non-React `apiClient` can attach it as a Bearer header.
- **Data**: TanStack Query over `apiClient` (`lib/apiClient.ts`), which prepends `/api/v1` and unwraps the success/error envelope into typed results, throwing `ApiClientError` (carrying the stable `code`) on failure. Per-domain hooks/clients live in `lib/` (`agents.ts`, `flows.ts`, `memory.ts`, etc.), with `.test.ts` siblings.
- **Canvas**: React Flow (`@xyflow/react`) under `components/flow/` — `FlowCanvas`, `AgentNode`, `AgentPalette`, `FlowToolbar`, save/load dialogs. Graph ↔ persistence mapping is in `lib/flowGraph.ts` / `lib/useFlowGraph.ts`.
- **SSE**: `hooks/useEventSource.ts` wraps native `EventSource` for streaming endpoints.
- Tests are colocated `*.test.tsx`/`*.test.ts` (vitest + Testing Library + jsdom, `src/test/setup.ts`).

## Feature workflow

Work is organized as numbered features `F01`–`F09`, each with `docs/FXX-*/spec.md` and `plan.md`. The `implement-feature` skill (`.agents/skills/`) reads a feature's spec + plan, implements it phase by phase, and commits **one commit per phase** on the current branch — matching the existing history (`feat(F08): stage N — ...`). Code comments reference the feature that introduced them (e.g. `// ... (F03)`), which is a useful way to trace why something exists.

Other skills under `.agents/skills/`: `spec-writer`, `prd-writer`, `rust-best-practices`, `vercel-react-best-practices`.
