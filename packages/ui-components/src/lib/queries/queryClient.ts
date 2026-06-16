import { browser } from "$app/environment";
import { QueryClient } from "@tanstack/svelte-query";

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      enabled: browser,
    },
  },
});

export const invalidateTanstackQueries = async (
  queryClient: QueryClient,
  queryKey: string[],
) => {
  try {
    await queryClient.invalidateQueries({
      queryKey,
      refetchType: "all",
      exact: false,
    });
  } catch {
    throw new Error("Failed to refresh data.");
  }
};
