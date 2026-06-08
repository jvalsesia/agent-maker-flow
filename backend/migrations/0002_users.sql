-- F02 Authentication & Access Control: local user identity, keyed by the Clerk
-- user id and JIT-upserted on the first authenticated request. FK target for
-- per-user record scoping in later features (F04, F05, F07, F08).
CREATE TABLE users (
    id         TEXT PRIMARY KEY,
    email      TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
