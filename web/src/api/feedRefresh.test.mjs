import assert from "node:assert/strict";
import { setImmediate } from "node:timers/promises";
import test from "node:test";
import { InfiniteQueryObserver, QueryClient, VUE_QUERY_CLIENT } from "@tanstack/vue-query";
import { createSSRApp, effectScope, nextTick, ref } from "vue";
import { FEED_PAGES_KEY, useRefreshFeedOnSync } from "./feedRefresh.ts";

const queryKey = [...FEED_PAGES_KEY, { limit: 50 }];

function setup(t, run) {
  const client = new QueryClient();
  const app = createSSRApp({});
  const scope = effectScope();
  const meta = ref(run === undefined ? undefined : successfulSync(run));

  app.provide(VUE_QUERY_CLIENT, client);
  app.runWithContext(() => scope.run(() => useRefreshFeedOnSync(meta)));
  t.after(() => {
    scope.stop();
    client.clear();
  });

  return { client, meta };
}

function successfulSync(run) {
  return { lastRunStatus: "ok", lastRunAt: run, lastSuccessfulRunAt: run };
}

test("the first successful sync refreshes a feed that observed a confirmed absence of sync", async (t) => {
  const { client, meta } = setup(t, undefined);

  client.setQueryData(queryKey, { pages: [], pageParams: [] });
  meta.value = { lastRunStatus: "never", lastRunAt: null, lastSuccessfulRunAt: null };
  await nextTick();
  assert.equal(client.getQueryState(queryKey).isInvalidated, false);

  meta.value = successfulSync("2026-11-20T12:00:00Z");
  await nextTick();
  assert.equal(client.getQueryState(queryKey).isInvalidated, true);
});

test("sync waits for pagination and then refreshes both pages", async (t) => {
  const { client, meta } = setup(t, "2026-11-20T12:00:00Z");
  const requested = [];
  const nextPage = Promise.withResolvers();

  const observer = new InfiniteQueryObserver(client, {
    queryKey,
    initialPageParam: "",
    getNextPageParam: (lastPage) => lastPage.nextCursor,
    queryFn: async ({ pageParam }) => {
      requested.push(pageParam);

      if (pageParam === "cursor") await nextPage.promise;

      return { nextCursor: pageParam === "" ? "cursor" : undefined };
    },
  });

  t.after(observer.subscribe(() => {}));
  await observer.refetch();

  const pagination = observer.fetchNextPage();

  try {
    meta.value = successfulSync("2026-11-20T12:15:00Z");
    await setImmediate();
    assert.deepEqual(requested, ["", "cursor"]);
  } finally {
    nextPage.resolve();
  }

  await pagination;
  await setImmediate();
  assert.deepEqual(requested, ["", "cursor", "", "cursor"]);
  assert.deepEqual(observer.getCurrentResult().data.pageParams, ["", "cursor"]);
});
