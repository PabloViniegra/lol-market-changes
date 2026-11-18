import type { PlayerResponse } from "@lol/shared";
import { useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { getJson } from "./client";

export function usePlayer(name: MaybeRefOrGetter<string>) {
  return useQuery({
    queryKey: computed(() => ["player", toValue(name)]),
    queryFn: () => getJson<PlayerResponse>(`/api/players/${encodeURIComponent(toValue(name))}`),
  });
}
