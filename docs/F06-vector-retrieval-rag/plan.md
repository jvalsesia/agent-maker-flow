# Implementation Plan: Vector Retrieval (RAG)

**Prerequisites:**
- Backend: Rust + Axum 0.8, sqlx 0.8 (Postgres + pgvector), the `pgvector` crate, running PostgreSQL and Redis, a LiteLLM proxy exposing at least one embedding-mode model.
- F03 gateway in place: `GatewayClient.embed()` in `AppState`.
- F05 in place: `memory_records` table (`embedding vector`, `embedding_model`), `agent_semantic_profiles`, `user_embedding_settings`, and the `memory::settings` read functions (`get_embedding_model`, `get_semantic_profile`).
- F04 agents available so an `agent_id` and its `top_k` override exist.
- No database migration and no new HTTP routes are introduced by this feature.

## Stage 1: Retrieval Service

**1. Retrieval Types** - Add the `retrieval` module under `memory` with the `RetrievedRecord`, `RetrievalOutcome`, and `RetrievalStatus` types described in the spec's Internal Service Contract, and declare/re-export the module from `memory/mod.rs`.

**2. Model Resolution & Cosine Search** - Implement embedding-model resolution (per-agent semantic profile override, else the user's global model) and the owner-scoped cosine query over `memory_records` filtered to the resolved model, plus the companion mismatched-model count. Follow the spec's Data Model query shapes.

**3. Retrieve Orchestration** - Implement `retrieve()` to clamp Top-K, short-circuit on `top_k = 0`, embed the prompt via the F03 gateway, run the search, and assemble the structured outcome — capturing every internal failure (no model, embedding error, search error) as a flagged `Skipped` status so the caller never sees an error. Reference the spec's behavioral contract.

**4. Unit Tests** - Add inline `#[cfg(test)]` tests for the Top-K clamp and the skip-reason mapping paths.

## Stage 2: Integration Tests

**5. Test Harness** - Add `backend/tests/retrieval_test.rs` reusing the gateway/memory harness, with a mock `/embeddings` that returns a vector encoded from the input and a way to seed records with known vectors, so cosine ranking is deterministic; soft-skip without a database.

**6. Retrieval Coverage** - Cover ranking by cosine similarity within Top-K, `top_k = 0` disabling retrieval, no-model and embedding-failure skip outcomes, mismatched-model exclusion with the reported count, per-agent profile model overriding the global model, and owner scoping — mapping to the spec's Testing Strategy and the PRD acceptance and cross-feature criteria.
