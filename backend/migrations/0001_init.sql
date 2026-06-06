-- Enable pgvector for embedding storage and similarity search (used by F05, F06)
CREATE EXTENSION IF NOT EXISTS vector;

-- Ensure UUID generation is available for primary keys across features
CREATE EXTENSION IF NOT EXISTS pgcrypto;
