import type { MetaResponse } from "@lol/shared";
import { useQuery } from "@tanstack/vue-query";

async function fetchMeta(): Promise<MetaResponse> {
  const response = await fetch("/api/meta");

  if (!response.ok) throw new Error(`GET /api/meta failed with ${response.status}`);

  return response.json();
}

export function useMeta() {
  return useQuery({ queryKey: ["meta"], queryFn: fetchMeta });
}
