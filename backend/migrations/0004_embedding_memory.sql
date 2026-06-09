-- F05 Embedding & Semantic Memory Configuration: per-user embedding model
-- selection, optional per-agent semantic profile overrides, and a store of
-- text "memory records" embedded via the F03 gateway and persisted as pgvector
-- vectors. Consumed by F06 (Vector Retrieval / RAG). The `vector` and
-- `pgcrypto` extensions are already enabled in 0001_init.sql.

-- Global per-user embedding model (one row per user, upserted).
CREATE TABLE user_embedding_settings (
    user_id         TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    embedding_model TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Optional per-agent override of the embedding model and memory scope (F06).
CREATE TABLE agent_semantic_profiles (
    agent_id        UUID PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    embedding_model TEXT NOT NULL,
    memory_scope    TEXT NOT NULL DEFAULT 'all',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ix_agent_semantic_profiles_user ON agent_semantic_profiles (user_id);

-- Embedded memory records. The vector dimension varies by model, so the column
-- is unbounded `vector` and the model is stored alongside; F06 filters by
-- (user_id, embedding_model) before comparing vectors. No ANN index here —
-- F06 adds one with its retrieval query shape.
CREATE TABLE memory_records (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    text            TEXT NOT NULL CHECK (char_length(text) <= 8000),
    embedding       VECTOR NOT NULL,
    embedding_model TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ix_memory_records_user ON memory_records (user_id);
CREATE INDEX ix_memory_records_user_model ON memory_records (user_id, embedding_model);
