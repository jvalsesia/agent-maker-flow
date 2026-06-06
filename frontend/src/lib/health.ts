import { useQuery } from "@tanstack/react-query";

import { apiGet } from "./apiClient";

export interface HealthStatus {
  service: string;
  database: string;
  cache: string;
  pgvector: boolean;
}

/** Query the backend health endpoint. */
export function useHealth() {
  return useQuery({
    queryKey: ["health"],
    queryFn: () => apiGet<HealthStatus>("/health"),
  });
}
