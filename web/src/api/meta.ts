import type { MetaResponse } from "@lol/shared";
import { useQuery } from "@tanstack/vue-query";
import { getJson } from "./client";

export function useMeta() {
  return useQuery({ queryKey: ["meta"], queryFn: () => getJson<MetaResponse>("/api/meta") });
}
