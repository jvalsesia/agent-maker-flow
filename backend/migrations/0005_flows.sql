-- F08 Flow Persistence: named, per-user saved flows. Stores the complete F07
-- FlowGraph (nodes with agent references + positions, edges, root) verbatim in a
-- jsonb column so it round-trips for F07 reload and F09 execution with no
-- transformation. Per-user scoped via the FK to `users`; name uniqueness is
-- case-insensitive. This is the project's first jsonb column.
CREATE TABLE flows (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id   TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       VARCHAR(80) NOT NULL,
    graph      JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Enforce name uniqueness per user, case-insensitive.
CREATE UNIQUE INDEX ux_flows_owner_name ON flows (owner_id, lower(name));

-- Fast per-user listing.
CREATE INDEX ix_flows_owner ON flows (owner_id);
