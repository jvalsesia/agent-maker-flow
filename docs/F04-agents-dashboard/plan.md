# Implementation Plan: F04 Agents Dashboard

**Prerequisites:**
- Backend: Rust + Axum 0.8, sqlx 0.8 (Postgres, uuid + chrono features), running PostgreSQL with `pgcrypto` and `pgvector` (from `0001_init.sql`), Redis.
- F02 auth in place: `require_auth` middleware, `AuthUser` extractor, `ensure_owner`, `users` table (`0002_users.sql`).
- F03 gateway in place: `GatewayClient` in `AppState` with `list_models`/`list_providers`.
- Frontend: React + Vite + TypeScript, `@tanstack/react-query`, existing `apiClient`, `useProviders`/`useModels` hooks.
- Environment: `DATABASE_URL`, `REDIS_URL`, gateway base URL configured as for F03.

## Stage 1: Data Model and Persistence

**1. Agents Migration** - Add the `agents` migration creating the table owned by the user, with the per-user unique name index, the owner lookup index, and the range constraints. Follow the spec's Data Model section for exact columns and indexes.

**2. Agent Module and Repository** - Create the `agents` backend module with the `Agent` and request-body types and a sqlx repository that inserts, lists, fetches, updates, and deletes rows, all scoped by `owner_id`. Declare the module in `lib.rs`.

## Stage 2: Service, Errors, and Endpoints

**3. Error Variants** - Extend `AppError` with the validation and conflict variants and their stable codes and HTTP statuses so the agents flow renders the platform error envelope consistently.

**4. Agent Service** - Implement the service layer that validates every field per the PRD rules, enforces per-user name uniqueness with a friendly message, validates the provider/model pair against the F03 gateway catalog, and applies the ownership guard before mutating or returning a record.

**5. Agent Routes** - Add the five protected handlers (create, list, get, update, delete) that pull the authenticated user, delegate to the service, and return the success envelope; mount them on the protected router in `routes/mod.rs`.

**6. Backend Integration Tests** - Add the agents integration test suite using the existing gateway/auth test harness, covering CRUD, validation, uniqueness, provider/model checks, defaults, and cross-user isolation per the spec's Testing Strategy.

## Stage 3: Frontend Data Layer

**7. API Client Verbs** - Extend the shared `apiClient` with the post, put, and delete helpers built on the existing request/envelope handling.

**8. Agent Hooks and Validation** - Add the `agents` data module with the agent type and the react-query list and create/update/delete mutation hooks (invalidating the agents query on success), plus the pure client-side validation helpers that mirror the server field rules. Include their unit tests.

## Stage 4: Agents Dashboard UI

**9. Agent Form** - Build the create/edit/duplicate form that reuses `useProviders`/`useModels` for the provider and provider-filtered model dropdowns (model disabled until a provider is chosen, incompatible model cleared on provider change), shows inline field-level validation, prefills "(copy)" in duplicate mode, and maps API error codes to the relevant fields.

**10. Agent List and Delete Confirmation** - Build the registry list showing each agent's name, provider, model, recent-N, and top-K with edit, duplicate, and delete actions, and the delete confirmation dialog that warns which flows reference the agent (referenced-flows seam left empty until F08 per the spec).

**11. Agents Page Composition** - Replace the placeholder Agents page to compose the list, form, and delete dialog, managing create/edit/duplicate mode, and add the component tests for the form's provider/model dependency and inline validation behavior.
