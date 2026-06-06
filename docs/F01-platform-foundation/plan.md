# Implementation Plan: Platform Foundation

**Prerequisites:**
- Rust toolchain (stable) and Cargo; `sqlx-cli` for migration management.
- Node.js (LTS) and a package manager (npm/pnpm) for the Vite frontend.
- Docker + Docker Compose for the local PostgreSQL (pgvector) and Redis stack.
- Key dependencies: `axum` 0.8.9, `tokio`, `sqlx` (postgres + migrate features), `pgvector`, `redis` + `deadpool-redis`, `tower-http`, `tracing` + `tracing-subscriber`, `thiserror`, `anyhow`, `serde`; `react`, `react-dom`, `react-router-dom`, `@tanstack/react-query`, `vite`, `typescript`, `vitest`, `@testing-library/react`.
- Environment variables: database URL, Redis URL, server bind address, allowed frontend origin (see `backend/.env.example`).

### Stage 1: Local Infrastructure & Project Scaffolding

**1. Local Service Stack** - Create the Docker Compose definition that runs a pgvector-enabled PostgreSQL and a Redis instance with mapped ports and persistent volumes, so the backend has reachable dependencies in development. Reference the spec's Local infra component.

**2. Backend Crate Scaffold** - Initialize the `backend/` Cargo project and declare all foundation dependencies in the manifest. Establish the module layout (config, state, error, db, cache, telemetry, routes, sse) as described in the spec's Component Overview.

**3. Frontend Project Scaffold** - Initialize the `frontend/` Vite + React + TypeScript project, declare routing, data, and testing dependencies, and configure the dev server proxy to the backend API. Reference the spec's Frontend components.

### Stage 2: Backend Foundation Services

**4. Configuration & Telemetry** - Implement the typed configuration layer that reads required settings from the environment and fails clearly when they are missing, and initialize structured logging. Reference the spec's `config.rs` and `telemetry.rs` responsibilities.

**5. Database & Cache Connectivity** - Implement PostgreSQL pool creation with automatic migration application and a connectivity/extension verification, plus Redis pool creation with a connectivity check. Author the bootstrap migration that enables the required extensions per the spec's Data Model.

**6. Shared State & Error Envelope** - Implement the cloneable application state that carries the database pool, cache pool, and configuration, and the application error type that renders the standard JSON error envelope across all handlers. Reference the spec's `state.rs` and `error.rs`.

**7. Startup Boot Sequence** - Wire the entrypoint to initialize telemetry and configuration, build state, run the boot connectivity checks and migrations, and only then bind the listener and serve — aborting with a dependency-named log if any check fails. Reference the spec's boot fail-fast decision.

### Stage 3: API & Streaming Base

**8. REST Router & Health Endpoint** - Assemble the versioned `/api/v1` router with CORS and tracing layers, reserve mount points for later features, and implement the health endpoint that aggregates service, database, cache, and pgvector status into the success envelope. Reference the spec's API Contracts.

**9. SSE Heartbeat Base** - Implement the heartbeat streaming endpoint that emits periodic named ping events, establishing the SSE event and encoding contract that execution-streaming features will reuse. Reference the spec's SSE base contract.

### Stage 4: Frontend Shell & Clients

**10. API & Query Clients** - Implement the base-URL REST client that parses the standard success/error envelopes into typed results, configure the TanStack Query client, and expose the health query hook. Reference the spec's `apiClient`, `queryClient`, and `health` components.

**11. Application Shell & Routing** - Implement the bootstrap that wraps the app in the query and router providers (reserving the Clerk provider slot for F02), the persistent shell with navigation, and the client-side routes for the Agents and Flows workspaces with a sensible default redirect. Reference the spec's `main.tsx`, router, `AppShell`, `NavBar`, and placeholder pages.

**12. SSE Hook Scaffolding** - Implement the reusable EventSource hook that manages connection lifecycle, exposes connection state and the latest event, and provides reconnection scaffolding for later streaming features. Reference the spec's `useEventSource` component.
