import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { apiDelete, apiGet, apiPost, apiPut } from "./apiClient";

/** Maximum length of a memory record's source text (characters). */
export const MEMORY_TEXT_MAX = 8000;

/** A stored memory record (the raw embedding vector is never sent to clients). */
export interface MemoryRecord {
  id: string;
  text: string;
  embedding_model: string;
  /** The agent this record is scoped to, or null for a global record (F06). */
  agent_id: string | null;
  char_count: number;
  created_at: string;
  updated_at: string;
}

/** Fields a memory record write accepts: the text plus an optional agent scope. */
export interface MemoryRecordInput {
  text: string;
  /** Scope the record to one agent (memory_scope="own"), or null for global. */
  agentId: string | null;
}

interface MemoryListResponse {
  records: MemoryRecord[];
  models_in_use: string[];
}

export const memoryKey = ["memory"] as const;

/** Query the caller's memory records + distinct models in use (`GET /memory`). */
export function useMemoryRecords() {
  return useQuery({
    queryKey: memoryKey,
    queryFn: () => apiGet<MemoryListResponse>("/memory"),
  });
}

/** Create a memory record (`POST /memory`); invalidates the memory query. */
export function useCreateMemoryRecord() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ text, agentId }: MemoryRecordInput) =>
      apiPost<MemoryRecord>("/memory", { text, agent_id: agentId }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: memoryKey }),
  });
}

/** Update (re-embed) a memory record (`PUT /memory/{id}`); invalidates memory. */
export function useUpdateMemoryRecord() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, text, agentId }: MemoryRecordInput & { id: string }) =>
      apiPut<MemoryRecord>(`/memory/${id}`, { text, agent_id: agentId }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: memoryKey }),
  });
}

/** Delete a memory record (`DELETE /memory/{id}`); invalidates the memory query. */
export function useDeleteMemoryRecord() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => apiDelete<{ id: string }>(`/memory/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: memoryKey }),
  });
}
