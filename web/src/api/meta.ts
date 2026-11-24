import type { MetaResponse } from "@lol/shared";
import { useQuery } from "@tanstack/vue-query";
import { getJson, isClientError } from "./client";
import { queryKeys } from "./keys";

export function useMeta() {
  return useQuery({
    queryKey: queryKeys.meta,
    queryFn: ({ signal }) => getJson<MetaResponse>("/api/meta", {}, signal),
    staleTime: 30_000,
    refetchInterval: (query) => {
      if (query.state.status !== "error") return 5 * 60_000;

      return isClientError(query.state.error) ? false : 30_000;
    },
  });
}
