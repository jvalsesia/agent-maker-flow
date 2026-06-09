import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { apiDelete, apiGet, apiPost, apiPut } from "./apiClient";

/** Maximum length of a memory record's source text (characters). */
export const MEMORY_TEXT_MAX = 8000;

/** A stored memory record (the raw embedding vector is never sent to clients). */
export interface MemoryRecord {
  id: string;
  text: string;
  embedding_model: string;
  char_count: number;
  created_at: string;
  updated_at: string;
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
    mutationFn: (text: string) => apiPost<MemoryRecord>("/memory", { text }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: memoryKey }),
  });
}

/** Update (re-embed) a memory record (`PUT /memory/{id}`); invalidates memory. */
export function useUpdateMemoryRecord() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, text }: { id: string; text: string }) =>
      apiPut<MemoryRecord>(`/memory/${id}`, { text }),
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
