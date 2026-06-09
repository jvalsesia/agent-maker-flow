# Implementation Plan: Embedding & Semantic Memory Configuration

**Prerequisites:**
- Existing F01–F03 backend (Axum 0.8.9, sqlx 0.8 with PostgreSQL + pgvector, deadpool-redis), F02 auth middleware + `ensure_owner`, and the F03 `GatewayClient` (`embed`, `list_models`) carried in `AppState`.
- New backend dependency: `pgvector = { version = "0.4", features = ["sqlx"] }` (binds `Vec<f32>` to the `vector` column).
- PostgreSQL with the `pgvector` extension — already enabled by `0001_init.sql` (no new extension migration).
- A reachable `DATABASE_URL`, `REDIS_URL`, and a running LiteLLM proxy exposing at least one embedding-mode model for integration testing.
- F04 `agents` table available before migration `0003` runs (same-batch dependency; see spec Assumption 1).
- Frontend: React 18 + Vite, TanStack Query, Clerk; existing `apiClient`, `models.ts` hooks, and the protected app-shell router.

## Stage 1: Data Model & Migration

**1. Embedding & Memory Schema** - Add migration `0003_embedding_memory.sql` creating the `user_embedding_settings`, `agent_semantic_profiles`, and `memory_records` tables with their owner foreign keys, the per-agent FK to F04's `agents` table, the vector column, the size CHECK, and the listed indexes. Reuse the pgvector extension already enabled in the initial migration. See the spec Data Model section.

**2. pgvector Crate** - Add the `pgvector` crate to the backend manifest so embedding vectors encode/decode through sqlx. Reference the spec Technical Decisions.

## Stage 2: Backend Repository & Gateway Orchestration

**3. Domain Types** - Add the `memory` module with DTOs for the embedding setting, semantic profile, and memory record, plus the shared 8,000-character limit constant. Mirror the existing `FromRow`/`Serialize` conventions from the users and gateway types.

**4. Settings Repository** - Implement upsert/get for the global embedding setting and upsert/get/delete for the per-agent semantic profile, validating any chosen model against the F03 embedding-model catalog before persisting. Follow the spec API and Data Model.

**5. Memory Record Repository** - Implement embed-on-save: resolve the user's global model, call the F03 gateway to embed the text, and insert the record only on success so a failed embedding is never stored; add owner-scoped list (with distinct models-in-use), update (re-embed), and delete. Reference the spec Decisions and Error Handling.

**6. Error Variants** - Extend the application error enum with the oversized-record and no-global-model cases and map them to the codes/statuses in the spec, reusing the existing gateway error variants for embed failures and unreachable proxy.

## Stage 3: Backend Endpoints

**7. Settings & Profile Endpoints** - Add the protected handlers for getting/setting the global embedding model and getting/setting/deleting the per-agent semantic profile, returning the platform success/error envelope. See the spec API Contracts.

**8. Memory Endpoints** - Add the protected handlers for listing, creating, updating, and deleting memory records, enforcing the size guard and owner guard and surfacing embed failures as the documented errors. Mount all new routes on the protected router. Reference the spec API Contracts.

## Stage 4: Frontend Settings Panel

**9. Write Client Helpers** - Extend the REST client with post/put/delete helpers that reuse the existing envelope and error handling. Reference the spec Component Overview.

**10. Settings & Memory Hooks** - Add TanStack Query hooks for the embedding setting, the semantic profile, and memory-record CRUD, following the existing `models.ts` hook conventions and invalidating the relevant queries on mutation. See the spec Component Overview.

**11. Settings Panel UI** - Build the settings page composing the embedding-model dropdown (sourced from the F03 catalog filtered to embedding-mode models, with the model-changed advisory warning) and the memory-record list/form with the size counter and the embedding-in-progress → stored/ready states. Add the protected `/settings` route and a NavBar link. Reference the spec Architecture Impact and Experience mapping.
