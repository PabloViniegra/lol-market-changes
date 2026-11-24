import type { MetaResponse } from "@lol/shared";
import { useQueryClient } from "@tanstack/vue-query";
import { computed, type Ref, watch } from "vue";

let currentRun: string | null = null;

export function lastSyncedRun() {
  return currentRun;
}

export function useRefreshOnSync(meta: Ref<MetaResponse | undefined>) {
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

      if (run !== null) currentRun = run;

      // Queries cached before the first meta response were fetched without a sync key, so the CDN
      // may have answered them from before the last ingest.
      if (run !== null && run !== lastSeen) {
        for (const query of queryClient.getQueryCache().findAll({ queryKey: ["data"] })) {
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
