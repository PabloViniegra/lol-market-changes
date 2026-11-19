import type { FlowsResponse, StatsResponse, StatsWindow } from "@lol/shared";
import { keepPreviousData, useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { getJson } from "./client";

export function useStats(window: MaybeRefOrGetter<StatsWindow>) {
  return useQuery({
    queryKey: computed(() => ["stats", toValue(window)]),
    queryFn: () => getJson<StatsResponse>("/api/stats", { window: toValue(window) }),
  });
}

export function useFlows(
  level: MaybeRefOrGetter<FlowsResponse["level"]>,
  window: MaybeRefOrGetter<StatsWindow>,
) {
  return useQuery({
    queryKey: computed(() => ["flows", toValue(level), toValue(window)]),
    queryFn: () =>
      getJson<FlowsResponse>("/api/flows", { level: toValue(level), window: toValue(window) }),
    placeholderData: keepPreviousData,
  });
}
