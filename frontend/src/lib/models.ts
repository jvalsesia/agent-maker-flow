import { useQuery } from "@tanstack/react-query";

import { apiGet } from "./apiClient";

/** A provider discovered from the gateway catalog (F03). */
export interface Provider {
  id: string;
  model_count: number;
}

/** A model offered by a provider. `mode` is `"chat"` or `"embedding"`. */
export interface Model {
  id: string;
  label: string;
  mode: string;
}

interface ProvidersResponse {
  providers: Provider[];
}

interface ModelsResponse {
  provider: string;
  models: Model[];
}

/** Query the provider catalog (`GET /providers`). */
export function useProviders() {
  return useQuery({
    queryKey: ["providers"],
    queryFn: () => apiGet<ProvidersResponse>("/providers").then((data) => data.providers),
  });
}

/**
 * Query the models for a provider (`GET /providers/{provider}/models`).
 * Disabled until a provider is selected, mirroring the F04/F05 "pick provider,
 * then model" flow.
 */
export function useModels(provider: string | null) {
  return useQuery({
    queryKey: ["models", provider],
    queryFn: () =>
      apiGet<ModelsResponse>(`/providers/${encodeURIComponent(provider ?? "")}/models`).then(
        (data) => data.models,
      ),
    enabled: provider != null && provider.length > 0,
  });
}
