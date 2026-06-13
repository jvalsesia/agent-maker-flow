# Agent Maker Flow

A node-based, visual orchestrator for multi-agent LLM pipelines. Define reusable
**agents** in a registry, wire them into a directed **flow** on a React Flow
canvas, mark a root, and run the graph against a chat prompt. Each agent's output
forwards into its downstream nodes, and the whole run streams back into a chat
monitor over SSE — so the otherwise opaque internals of a multi-step agent run
become observable node by node.

Instead of hand-coding prompt chains, reordering, re-modeling, and re-prompting a
pipeline is a drag-and-drop or form edit. See [`PRD.md`](PRD.md) and
[`project-intent.md`](project-intent.md) for the full product vision, and
[`usage.md`](usage.md) for how the semantic-memory feature works end to end.

## Highlights

- **Visual DAG editor** — drag agents onto a canvas, connect output→input ports,
  designate a root. Cycles and self-loops are rejected so the graph stays a valid DAG.
- **Reusable agent registry** — per-agent provider, model, preamble, system prompt,
  recent-N (history depth), and top-K (retrieval breadth).
- **Unified LLM gateway** — every completion and embedding call routes through a
  single LiteLLM proxy, with provider/model discovery and a Redis exact-match
  completion cache (identical requests aren't re-billed).
- **Retrieval-augmented context (RAG)** — memory records are embedded and stored in
  PostgreSQL via `pgvector`; at execution the prompt is embedded and the top-K most
  similar records (by cosine similarity) are injected ahead of the forwarded input.
- **Real-time streaming monitor** — agent blocks light up idle → running →
  complete/error and stream intermediate + final output live via Server-Sent Events.
- **Per-user isolation** — Clerk authentication scopes every agent, flow, and memory
  record to its owner; another user's record returns `404`, never revealing its existence.

## Architecture

```
┌──────────────────────────┐         REST /api/v1  +  SSE        ┌──────────────────────────┐
│  Frontend (React + Vite) │ ───────────────────────────────────▶│   Backend (Rust + Axum)   │
│  • React Flow canvas     │                                      │  • DAG translate + execute │
│  • Chat / SSE monitor    │◀──────────────────────────────────── │  • owner-scoped sqlx       │
│  • Clerk auth            │           execution events           │  • Clerk JWT middleware    │
└──────────────────────────┘                                      └────────────┬──────────────┘
                                                                                │
                                              ┌─────────────────────────────────┼───────────────────┐
                                              ▼                                  ▼                   ▼
                                   ┌────────────────────┐          ┌─────────────────────┐  ┌──────────────┐
                                   │ LiteLLM proxy       │          │ PostgreSQL+pgvector │  │   Redis      │
                                   │ (all providers,     │          │ agents, flows,      │  │ cache +      │
                                   │  completions+embeds)│          │ memory vectors      │  │ usage counters│
                                   └────────────────────┘          └─────────────────────┘  └──────────────┘
```

- **Backend** — Rust [Axum](https://github.com/tokio-rs/axum) (v0.8) on Tokio. REST
  under `/api/v1` + SSE. Boot is fail-fast: connect + migrate Postgres, verify the
  `pgvector` extension, PING Redis, and warm the Clerk JWKS cache *before* binding
  the listener. Hand-written `sqlx` queries (no ORM), all owner-scoped.
- **Frontend** — React 18 + Vite + TypeScript SPA. React Router v6, TanStack Query
  over a thin `apiClient`, [`@xyflow/react`](https://reactflow.dev) for the canvas,
  `@clerk/clerk-react` for auth, and native `EventSource` for SSE streaming.
- **Gateway** — [LiteLLM](https://github.com/BerriAI/litellm) is the single path to
  every provider (OpenAI, Anthropic, Groq, Ollama, …). Provider API keys live in the
  LiteLLM container, not the backend. Add models in `litellm/config.yaml`.

The product is built as numbered features **F01–F10** (PRD scope) plus **F11**
(design-system / UI polish); each has a `spec.md` and `plan.md` under `docs/FXX-*/`.

## Repository layout

```
backend/      Rust Axum service — REST + SSE
  src/
    agents/ flows/ memory/ gateway/ auth/ runs/   feature modules
    routes/                                        HTTP handlers
    main.rs lib.rs state.rs error.rs              boot, app state, error envelope
  migrations/                                      NNNN_*.sql, applied at boot
  tests/                                           integration suite (live-infra, soft-skip)
frontend/     React + Vite SPA
  src/
    pages/ components/ components/flow/ lib/ hooks/
litellm/config.yaml   model catalog for the LiteLLM proxy
docs/F01..F11/        per-feature spec.md + plan.md
docker-compose.yml    postgres (pgvector), redis, litellm
```

## Prerequisites

- [Docker](https://www.docker.com/) + Docker Compose (for Postgres, Redis, LiteLLM)
- [Rust](https://www.rust-lang.org/) (stable, edition 2021)
- [Node.js](https://nodejs.org/) 18+ and npm
- A [Clerk](https://clerk.com/) application (publishable key + issuer)
- At least one LLM provider key (e.g. `OPENAI_API_KEY`) for the LiteLLM proxy

## Getting started

**1. Start infrastructure** (Postgres on `:5432`, Redis on `:6379`, LiteLLM on `:4000`):

```bash
docker compose up -d
```

**2. Configure environment.** Copy the examples and fill in the required values:

```bash
cp backend/.env.example backend/.env       # CLERK_ISSUER, CLERK_AUTHORIZED_PARTIES required
cp frontend/.env.example frontend/.env      # VITE_CLERK_PUBLISHABLE_KEY required
```

Provider keys (e.g. `OPENAI_API_KEY`) are read by the LiteLLM container — set them in
your shell or a root `.env` before `docker compose up`.

**3. Run the backend** (connects + migrates Postgres, PINGs Redis, warms JWKS, serves `:8080`):

```bash
cd backend
cargo run
```

**4. Run the frontend** (Vite dev server on `:5173`, proxies `/api` → `localhost:8080`):

```bash
cd frontend
npm install
npm run dev
```

Open http://localhost:5173, sign in with Clerk, and you'll land on the Agents
Dashboard. Create a couple of agents, switch to Flows, drag them onto the canvas,
wire them up, mark a root, and run a prompt.

## Common commands

**Backend** (`cd backend`):

```bash
cargo run                                    # boot the service
cargo test                                   # integration tests (soft-skip without live DB/Redis)
cargo test --test flows_test                 # one test file
cargo test --test flows_test create_flow     # one test by name
cargo clippy --all-targets
cargo fmt
```

**Frontend** (`cd frontend`):

```bash
npm run dev          # Vite dev server
npm test             # vitest run (jsdom)
npm test -- AgentForm  # one test file by name match
npm run build        # tsc --noEmit && vite build
npm run typecheck
```

## How a run works

1. You submit a chat prompt and press **Run Flow**. The prompt maps to the
   designated **Root Agent**.
2. The backend translates the canvas into a DAG and validates it is acyclic with
   exactly one root (`runs/graph.rs`).
3. Nodes execute in dependency order (`runs/engine.rs`): a node runs only after all
   its upstream nodes complete, and its output is forwarded as input to every
   connected downstream node.
4. For each node, the engine optionally retrieves the top-K most similar memory
   records (`memory/retrieval.rs`) and injects them ahead of the forwarded input,
   then calls the model through the LiteLLM gateway with the agent's full config.
5. Execution events (node started, partial output, completed, failed, run finished)
   stream over SSE; the monitor renders them live, and terminal-node output becomes
   the assistant turn.

See [`usage.md`](usage.md) for a deeper walkthrough of the memory / retrieval path.

## Testing notes

Backend integration tests in `backend/tests/` spin up the real Axum app against
`DATABASE_URL` / `REDIS_URL`. When those are absent or unreachable they print `SKIP:`
and return green — so `cargo test` passes without Docker but only *exercises* anything
when the stack is up. Auth-protected tests mint Clerk-style JWTs with an embedded test
keypair and a mock LiteLLM router, each using a freshly-nonced owner id so the suite is
rerunnable against a persistent DB.

## Out of scope (v1)

Team/workspace sharing and multi-user co-editing; conditional/branch routing and
cyclic flows; user-facing cost/usage dashboards; in-app LiteLLM provider/key
management; scheduled or programmatic flow execution; and automatic re-embedding when
the embedding model changes (the change is flagged, re-embedding is manual). See
[`PRD.md` §7](PRD.md) for the full list.
