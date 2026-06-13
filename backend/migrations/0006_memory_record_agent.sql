-- F06 carryover: link a memory record to an owning agent, so an agent whose
-- semantic profile sets memory_scope = 'own' retrieves only its own records.
-- Until now memory_records had no agent association, leaving 'own' inert (F06
-- retrieved across all of the user's records regardless of scope).
--
-- agent_id is nullable: a NULL means a "global" record, visible only under the
-- default 'all' scope. ON DELETE SET NULL so deleting an agent reverts its
-- tagged records to global rather than destroying the user's memory text.
ALTER TABLE memory_records
    ADD COLUMN agent_id UUID REFERENCES agents(id) ON DELETE SET NULL;

-- Backs the scope = 'own' retrieval filter (user_id, agent_id, embedding_model).
CREATE INDEX ix_memory_records_user_agent_model
    ON memory_records (user_id, agent_id, embedding_model);
