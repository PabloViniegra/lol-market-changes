import type { FeedResponse } from "@lol/shared";
import { useInfiniteQuery, useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { getData, type QueryParams } from "./client";
import { queryKeys } from "./keys";

export function useFeed(
  params: MaybeRefOrGetter<QueryParams>,
  enabled: MaybeRefOrGetter<boolean> = true,
) {
  return useQuery({
    enabled,
    queryKey: computed(() => queryKeys.feed.list(toValue(params))),
    queryFn: ({ queryKey: [, , filters], signal }) =>
      getData<FeedResponse>("/api/feed", filters, signal),
  });
}

export function useFeedPages(params: MaybeRefOrGetter<QueryParams>) {
  return useInfiniteQuery({
    queryKey: computed(() => queryKeys.feed.pages(toValue(params))),
    queryFn: ({ queryKey: [, , filters], pageParam, signal }) =>
      getData<FeedResponse>("/api/feed", { ...filters, cursor: pageParam }, signal),
    initialPageParam: "",
    getNextPageParam: (lastPage) => lastPage.nextCursor ?? undefined,
  });
}
