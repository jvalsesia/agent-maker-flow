# Memory Usage Guide

How the **Memory** feature in Settings is used. Short version: it's a per-user
knowledge store that gets semantically searched and injected into agent prompts
at runtime (RAG).

## The setup (Settings → Memory)

The Settings page (`frontend/src/pages/SettingsPage.tsx`) gives you three things:

1. **Pick a global embedding model** (e.g. `text-embedding-3-small`). This is
   required *before* you can save any memory — records can't be embedded
   without it.
2. **Add memory records** — free text up to 8,000 chars, each optionally
   **scoped to one agent** or left global ("All agents").
3. **List / edit / delete** existing records.

When you click *Add record*, the backend doesn't just store text — it calls the
LiteLLM gateway to **embed** the text into a vector and saves both together
(`memory/store.rs`). If embedding fails, nothing is stored.

```
POST /api/v1/memory
{ "text": "Return policy: 30 days for unopened items.", "agent_id": null }
```

`agent_id: null` = visible to every agent. A UUID = private to that one agent.

## How it's consumed at runtime

When a flow runs, **each agent node** does a retrieval step before calling the
LLM (`runs/engine.rs` → `memory/retrieval.rs`):

1. Embed the incoming prompt with the same model.
2. Cosine-search your memory records for the closest `top_k` (an agent setting,
   default 5, `0` disables it entirely):

   ```sql
   SELECT text, 1 - (embedding <=> $prompt_vec) AS score
   FROM memory_records
   WHERE user_id = $me AND embedding_model = $model
     AND ($scope_all OR agent_id = $this_agent)
   ORDER BY embedding <=> $prompt_vec
   LIMIT $top_k;
   ```

3. The top matches are **prepended to the user message**:

   ```
   System: <agent preamble + system prompt>

   User:
   Return policy: 30 days for unopened items.

   Customer: is my June 1st order still returnable?
   ```

The LLM answers with that context in front of it. Retrieval is best-effort — if
the model isn't set or embedding fails, the node just runs **without** memory
rather than erroring.

## Two scoping knobs worth knowing

- **Per-record scope** (set in Settings): global vs. tied to one agent.
- **Per-agent `memory_scope`** (the F06 semantic profile): `"all"` (default —
  agent sees all your records) or `"own"` (agent only sees records scoped to
  it). An agent can also override the embedding model here.

So `"own"` + agent-scoped records = a private knowledge base for that one agent;
the default `"all"` = a shared pool everyone draws from.

## Concrete example

**Goal:** a Support agent that knows your policies.

1. Settings → embedding model = `text-embedding-3-small`.
2. Add records (scoped to the "Support" agent, or global):
   - *"Returns accepted within 30 days for unopened items."*
   - *"Shipping is free over $50; otherwise $6 flat."*
   - *"We do not ship to PO boxes."*
3. Build a flow with the Support agent, `top_k = 5`.
4. Chat prompt: *"Can I return something I bought 3 weeks ago, and is reshipping
   free?"*

At execution the engine embeds that question, pulls the two most-similar records
(returns + shipping), prepends them, and the agent answers grounded in *your*
policies — even though none of it is in the system prompt.

## Caveat: changing the embedding model

If you switch embedding models after storing records, old records live in a
different vector space and **won't match** new queries (`models_in_use` drives
the UI warning). You'd need to re-save them under the new model.

## Key files

- `frontend/src/pages/SettingsPage.tsx` — Settings panel
- `frontend/src/components/MemoryRecordForm.tsx` / `MemoryRecordList.tsx` — UI
- `frontend/src/lib/memory.ts` / `embeddingSettings.ts` — client hooks
- `backend/src/memory/store.rs` — embed-on-save
- `backend/src/memory/retrieval.rs` — cosine search + scoping
- `backend/src/runs/engine.rs` — prompt assembly + injection
- `backend/src/routes/memory.rs` / `settings.rs` — endpoints
- `docs/F05-embedding-semantic-memory-configuration/spec.md` — settings spec
- `docs/F06-vector-retrieval-rag/spec.md` — retrieval spec
