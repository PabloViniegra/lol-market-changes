import type { FlowsResponse, StatsResponse, StatsWindow } from "@lol/shared";
import { keepPreviousData, useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { getData } from "./client";
import { queryKeys } from "./keys";

export function useStats(window: MaybeRefOrGetter<StatsWindow>) {
  return useQuery({
    queryKey: computed(() => queryKeys.stats(toValue(window))),
    queryFn: ({ queryKey: [, statsWindow], signal }) =>
      getData<StatsResponse>("/api/stats", { window: statsWindow }, signal),
  });
}

export function useFlows(
  level: MaybeRefOrGetter<FlowsResponse["level"]>,
  window: MaybeRefOrGetter<StatsWindow>,
) {
  return useQuery({
    queryKey: computed(() => queryKeys.flows(toValue(level), toValue(window))),
    queryFn: ({ queryKey: [, flowLevel, flowWindow], signal }) =>
      getData<FlowsResponse>("/api/flows", { level: flowLevel, window: flowWindow }, signal),
    placeholderData: keepPreviousData,
  });
}
