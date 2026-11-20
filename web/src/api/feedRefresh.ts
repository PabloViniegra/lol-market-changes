import type { MetaResponse } from "@lol/shared";
import { useQueryClient } from "@tanstack/vue-query";
import { computed, type Ref, watch } from "vue";

export const FEED_PAGES_KEY = ["feed-pages"] as const;

export function useRefreshFeedOnSync(meta: Ref<MetaResponse | undefined>) {
  const queryClient = useQueryClient();

  const lastSuccessfulRun = computed(() => {
    const latest = meta.value;

    if (latest === undefined) return undefined;

    return latest.lastSuccessfulRunAt ?? (latest.lastRunStatus === "ok" ? latest.lastRunAt : null);
  });

  let lastSeen: string | null | undefined;

  watch(
    lastSuccessfulRun,
    (run) => {
      if (run === undefined) return;

      if (run !== null && lastSeen !== undefined && run !== lastSeen) {
        for (const query of queryClient.getQueryCache().findAll({ queryKey: FEED_PAGES_KEY })) {
          const refresh = () =>
            queryClient.invalidateQueries(
              { queryKey: query.queryKey, exact: true },
              { cancelRefetch: false },
            );

          if (query.promise) {
            void query.promise.then(refresh, refresh);
          } else {
            void refresh();
          }
        }
      }

      lastSeen = run ?? lastSeen ?? null;
    },
    { immediate: true },
  );

  return lastSuccessfulRun;
}
