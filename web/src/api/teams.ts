import type { TeamResponse } from "@lol/shared";
import { useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { getJson } from "./client";

export function useTeam(name: MaybeRefOrGetter<string>) {
  return useQuery({
    queryKey: computed(() => ["team", toValue(name)]),
    queryFn: () => getJson<TeamResponse>(`/api/teams/${encodeURIComponent(toValue(name))}`),
  });
}
