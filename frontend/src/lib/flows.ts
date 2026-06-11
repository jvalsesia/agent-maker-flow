import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { apiDelete, apiGet, apiPatch, apiPost, apiPut } from "./apiClient";
import type { FlowGraph } from "./flowGraph";

/** A saved flow summary (F08): the list shape, without the graph. */
export interface FlowSummary {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

/** A saved flow with its full graph — the shape returned on open/save. */
export interface Flow extends FlowSummary {
  graph: FlowGraph;
}

/** Save (create) / re-save (update) request body. */
export interface FlowInput {
  name: string;
  graph: FlowGraph;
}

interface FlowsResponse {
  flows: FlowSummary[];
}

/** Query key for the caller's saved-flows list. */
export const flowsQueryKey = ["flows"] as const;

/** Query key for a single opened flow. */
export function flowQueryKey(id: string) {
  return ["flows", id] as const;
}

/** List the caller's saved flows (`GET /flows`), most recently updated first. */
export function useFlows() {
  return useQuery({
    queryKey: flowsQueryKey,
    queryFn: () => apiGet<FlowsResponse>("/flows").then((data) => data.flows),
  });
}

/**
 * Open a single flow with its full graph (`GET /flows/{id}`). Disabled until an
 * id is provided, so the page can hold `null` when nothing is open.
 */
export function useFlow(id: string | null) {
  return useQuery({
    queryKey: flowQueryKey(id ?? ""),
    queryFn: () => apiGet<Flow>(`/flows/${id}`),
    enabled: id != null,
  });
}

/** Save a new flow (`POST /flows`); invalidates the flows queries on success. */
export function useCreateFlow() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: FlowInput) => apiPost<Flow>("/flows", input),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: flowsQueryKey }),
  });
}

/** Re-save the open flow (`PUT /flows/{id}`); invalidates the flows queries. */
export function useUpdateFlow() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: FlowInput }) =>
      apiPut<Flow>(`/flows/${id}`, input),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: flowsQueryKey }),
  });
}

/** Rename a flow (`PATCH /flows/{id}`); invalidates the flows queries. */
export function useRenameFlow() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) =>
      apiPatch<FlowSummary>(`/flows/${id}`, { name }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: flowsQueryKey }),
  });
}

/** Delete a flow (`DELETE /flows/{id}`); invalidates the flows queries. */
export function useDeleteFlow() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => apiDelete<{ id: string }>(`/flows/${id}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: flowsQueryKey }),
  });
}
