import { QueryClient } from "@tanstack/react-query";

/** Shared TanStack Query client with conservative foundation defaults. */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 30_000,
      refetchOnWindowFocus: false,
    },
  },
});
