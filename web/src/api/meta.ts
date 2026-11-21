import type { MetaResponse } from "@lol/shared";
import { useQuery } from "@tanstack/vue-query";
import { getJson } from "./client";
import { queryKeys } from "./keys";

export function useMeta() {
  return useQuery({
    queryKey: queryKeys.meta,
    queryFn: ({ signal }) => getJson<MetaResponse>("/api/meta", {}, signal),
  });
}
