import type { FeedResponse } from "@lol/shared";
import { useInfiniteQuery, useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { getJson, type QueryParams } from "./client";
import { FEED_PAGES_KEY } from "./feedRefresh";

export function useFeed(
  params: MaybeRefOrGetter<QueryParams>,
  enabled: MaybeRefOrGetter<boolean> = true,
) {
  return useQuery({
    enabled,
    queryKey: computed(() => ["feed", toValue(params)] as const),
    queryFn: ({ queryKey: [, filters], signal }) =>
      getJson<FeedResponse>("/api/feed", filters, signal),
  });
}

export function useFeedPages(params: MaybeRefOrGetter<QueryParams>) {
  return useInfiniteQuery({
    queryKey: computed(() => [...FEED_PAGES_KEY, toValue(params)] as const),
    queryFn: ({ queryKey: [, filters], pageParam, signal }) =>
      getJson<FeedResponse>("/api/feed", { ...filters, cursor: pageParam }, signal),
    initialPageParam: "",
    getNextPageParam: (lastPage) => lastPage.nextCursor ?? undefined,
  });
}
