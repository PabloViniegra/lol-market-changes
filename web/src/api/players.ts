import type { PlayerResponse } from "@lol/shared";
import { useQuery, useQueryClient, type QueryFunctionContext } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { getData } from "./client";
import { queryKeys } from "./keys";

function fetchPlayer({
  queryKey: [, name],
  signal,
}: QueryFunctionContext<ReturnType<typeof queryKeys.player>>) {
  return getData<PlayerResponse>(`/api/players/${encodeURIComponent(name)}`, {}, signal);
}

export function usePlayer(name: MaybeRefOrGetter<string>) {
  return useQuery({
    queryKey: computed(() => queryKeys.player(toValue(name))),
    queryFn: fetchPlayer,
  });
}

export function usePrefetchPlayer() {
  const queryClient = useQueryClient();

  return (name: string) =>
    void queryClient.prefetchQuery({ queryKey: queryKeys.player(name), queryFn: fetchPlayer });
}
