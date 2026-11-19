import type { FeedResponse } from "@lol/shared";
import { useInfiniteQuery, useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { getJson, type QueryParams } from "./client";

export function useFeed(
  params: MaybeRefOrGetter<QueryParams>,
  enabled: MaybeRefOrGetter<boolean> = true,
) {
  return useQuery({
    enabled,
    queryKey: computed(() => ["feed", toValue(params)]),
    queryFn: () => getJson<FeedResponse>("/api/feed", toValue(params)),
  });
}

export function useFeedPages(params: MaybeRefOrGetter<QueryParams>) {
  return useInfiniteQuery({
    queryKey: computed(() => ["feed-pages", toValue(params)]),
    queryFn: ({ pageParam }: { pageParam: string | null }) =>
      getJson<FeedResponse>("/api/feed", { ...toValue(params), cursor: pageParam }),
    initialPageParam: null,
    getNextPageParam: (lastPage) => lastPage.nextCursor,
  });
}
