import type { MetaResponse } from "@lol/shared";
import { useQueryClient } from "@tanstack/vue-query";
import { computed, type Ref, watch } from "vue";

// Meta drives this refresh and the team catalog keeps its own long cache.
const SYNC_INDEPENDENT_ROOTS = new Set<unknown>(["meta", "teams"]);

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

      if (run !== null && lastSeen !== undefined && run !== lastSeen) {
        for (const query of queryClient.getQueryCache().findAll({
          predicate: ({ queryKey }) => !SYNC_INDEPENDENT_ROOTS.has(queryKey[0]),
        })) {
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
