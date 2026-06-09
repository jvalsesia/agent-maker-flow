-- F04 Agents Dashboard: reusable LLM behavior profiles (name, preamble, system
-- prompt, provider/model, recent-N, top-K) owned by a user. The system of
-- record consumed later by F07 (Flow Canvas) and F09 (Flow Execution Engine).
-- Per-user scoped via the FK to `users`; name uniqueness is case-insensitive.
CREATE TABLE agents (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name          VARCHAR(64) NOT NULL,
    preamble      VARCHAR(2000),
    system_prompt VARCHAR(32000) NOT NULL,
    provider      TEXT NOT NULL,
    model         TEXT NOT NULL,
    recent_n      INTEGER NOT NULL DEFAULT 10 CHECK (recent_n BETWEEN 0 AND 100),
    top_k         INTEGER NOT NULL DEFAULT 5  CHECK (top_k BETWEEN 0 AND 50),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Enforce name uniqueness per user, case-insensitive.
CREATE UNIQUE INDEX ux_agents_owner_name ON agents (owner_id, lower(name));

-- Fast per-user listing.
CREATE INDEX ix_agents_owner ON agents (owner_id);
