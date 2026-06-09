import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { apiDelete, apiGet, apiPost, apiPut } from "./apiClient";

/** A persisted agent (F04): a reusable LLM behavior profile. */
export interface Agent {
  id: string;
  name: string;
  preamble: string | null;
  system_prompt: string;
  provider: string;
  model: string;
  recent_n: number;
  top_k: number;
  created_at: string;
  updated_at: string;
}

/** Create/update request body — the editable fields of an agent. */
export interface AgentInput {
  name: string;
  preamble: string | null;
  system_prompt: string;
  provider: string;
  model: string;
  recent_n: number;
  top_k: number;
}

interface AgentsResponse {
  agents: Agent[];
}

/** Query key for the caller's agent registry. */
export const agentsQueryKey = ["agents"] as const;

/** List the caller's agents (`GET /agents`), ordered by name. */
export function useAgents() {
  return useQuery({
    queryKey: agentsQueryKey,
    queryFn: () => apiGet<AgentsResponse>("/agents").then((data) => data.agents),
  });
}

/** Create an agent (`POST /agents`); invalidates the agents query on success. */
export function useCreateAgent() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: AgentInput) => apiPost<Agent>("/agents", input),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: agentsQueryKey }),
  });
}

/** Update an agent (`PUT /agents/{id}`); invalidates the agents query. */
export function useUpdateAgent() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: AgentInput }) =>
      apiPut<Agent>(`/agents/${id}`, input),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: agentsQueryKey }),
  });
}

/** Delete an agent (`DELETE /agents/{id}`); invalidates the agents query. */
export function useDeleteAgent() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => apiDelete<{ id: string }>(`/agents/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: agentsQueryKey }),
  });
}
