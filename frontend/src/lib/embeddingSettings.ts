import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { apiDelete, apiGet, apiPut } from "./apiClient";

/** The global embedding setting (`null` until the user picks a model). */
export interface EmbeddingSetting {
  embedding_model: string | null;
}

/** A per-agent semantic profile overriding the global model + memory scope. */
export interface SemanticProfile {
  agent_id: string;
  embedding_model: string;
  memory_scope: string;
}

export interface SemanticProfileInput {
  embedding_model: string;
  memory_scope?: string;
}

export const embeddingSettingKey = ["embedding-setting"] as const;
export const semanticProfileKey = (agentId: string) => ["semantic-profile", agentId] as const;

/** Query the global embedding model (`GET /settings/embedding`). */
export function useEmbeddingSetting() {
  return useQuery({
    queryKey: embeddingSettingKey,
    queryFn: () => apiGet<EmbeddingSetting>("/settings/embedding"),
  });
}

/** Set the global embedding model (`PUT /settings/embedding`); invalidates it. */
export function useSetEmbeddingSetting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (embedding_model: string) =>
      apiPut<EmbeddingSetting>("/settings/embedding", { embedding_model }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: embeddingSettingKey }),
  });
}

/** Query an agent's semantic profile (`GET /agents/{id}/semantic-profile`). */
export function useSemanticProfile(agentId: string | null) {
  return useQuery({
    queryKey: ["semantic-profile", agentId],
    queryFn: () => apiGet<SemanticProfile>(`/agents/${agentId}/semantic-profile`),
    enabled: agentId != null && agentId.length > 0,
  });
}

/** Upsert an agent's semantic profile; invalidates that profile query. */
export function useSetSemanticProfile(agentId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: SemanticProfileInput) =>
      apiPut<SemanticProfile>(`/agents/${agentId}/semantic-profile`, input),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: semanticProfileKey(agentId) }),
  });
}

/** Delete an agent's semantic profile; invalidates that profile query. */
export function useDeleteSemanticProfile(agentId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => apiDelete<{ agent_id: string }>(`/agents/${agentId}/semantic-profile`),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: semanticProfileKey(agentId) }),
  });
}
