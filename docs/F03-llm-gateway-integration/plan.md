# Implementation Plan: LLM Gateway Integration

**Prerequisites:**
- F01 Platform Foundation (Redis pool in `AppState`, `/api/v1` router, error envelope) and F02 Authentication (`require_auth`, protected router) implemented.
- A reachable LiteLLM proxy. In dev, this plan adds a `litellm` service to docker-compose; provider API keys (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, etc.) must be available to that container.
- New backend dependency: `sha2` (`reqwest` already present).
- Environment variables: `LITELLM_BASE_URL` (e.g. `http://localhost:4000`) and optional `LITELLM_MASTER_KEY` (see `backend/.env.example`).

### Stage 1: Dev Gateway & Client Foundation

**1. LiteLLM Dev Service & Config** - Add a `litellm` service to docker-compose and author `litellm/config.yaml` declaring the supported providers' models and proxy settings, so the gateway is reachable locally. Document the new backend and provider environment variables in the env template. Reference the spec's Infrastructure components.

**2. Gateway Config, State & Errors** - Add the LiteLLM base URL and optional master key to the typed configuration, introduce the gateway error variants on the shared error type, and define the `GatewayClient` (HTTP client, base URL, master key, Redis pool) carried in the application state, built once at boot. Reference the spec's `config.rs`, `error.rs`, `state.rs`, `gateway/mod.rs`, and boot wiring.

### Stage 2: Provider & Model Discovery

**3. Catalog Discovery** - Implement discovery against the proxy's model-info endpoint: fetch the configured models, parse each model's provider and mode, and group them so the client can list providers and list models for a given provider. Reference the spec's `gateway/catalog.rs` and `gateway/types.rs`.

**4. Discovery Endpoints** - Implement the two protected REST handlers that return the provider catalog and the provider-filtered model list, wired into the protected router and returning the standard success/error envelopes. Reference the spec's API Contracts and `routes/providers.rs`.

### Stage 3: Completion, Embedding, Caching & Accounting

**5. Exact-Match Completion Cache** - Implement the cache layer that canonicalizes a completion request, derives a stable hash key, and reads/writes the cached result in Redis with the configured expiry, treating Redis errors as a miss. Reference the spec's Data Model and `gateway/cache.rs`.

**6. Usage & Cost Counters** - Implement the Redis counters that increment per-model and global token, request, and cost totals on each billed request, and a cache-hit counter, sourcing cost from the proxy's response. Reference the spec's Data Model and `gateway/usage.rs`.

**7. Completion Service** - Implement the internal completion function that checks the cache, calls the proxy on a miss, stores the result, increments the counters, and maps proxy/transport failures to the gateway error variants. Reference the spec's internal service contracts and `gateway/completion.rs`.

**8. Embedding Service** - Implement the internal embedding function that calls the proxy, returns the vector with its accounting, increments the counters, and maps failures so callers can degrade gracefully. Reference the spec's internal service contracts and `gateway/embedding.rs`.

### Stage 4: Frontend Catalog Client

**9. Provider/Model Hooks** - Implement the frontend catalog client with the provider and model types and the query hooks that fetch the provider list and the provider-scoped model list, with the model query disabled until a provider is selected, so F04/F05 can reuse them. Reference the spec's `lib/models.ts`.
