import assert from "node:assert/strict";
import { setImmediate } from "node:timers/promises";
import test from "node:test";
import { InfiniteQueryObserver, QueryClient, VUE_QUERY_CLIENT } from "@tanstack/vue-query";
import { createSSRApp, effectScope, nextTick, ref } from "vue";
import { lastSyncedRun, useRefreshOnSync } from "./syncRefresh.ts";

const queryKey = ["data", "feed", "pages", { limit: 50 }];

function setup(t, run) {
  const client = new QueryClient();
  const app = createSSRApp({});
  const scope = effectScope();
  const meta = ref(run === undefined ? undefined : successfulSync(run));

  app.provide(VUE_QUERY_CLIENT, client);
  app.runWithContext(() => scope.run(() => useRefreshOnSync(meta)));
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

test("the first meta response refreshes data fetched before it arrived", async (t) => {
  const { client, meta } = setup(t, undefined);

  client.setQueryData(queryKey, { pages: [], pageParams: [] });
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

test("sync refreshes data queries but not meta or the team catalog", async (t) => {
  const { client, meta } = setup(t, "2026-11-20T12:00:00Z");

  const keys = [
    ["data", "feed", "list", { limit: 6 }],
    ["data", "team", "t1"],
    ["data", "stats", "season"],
    ["meta"],
    ["catalog", "teams"],
  ];

  for (const key of keys) client.setQueryData(key, {});

  meta.value = successfulSync("2026-11-20T12:15:00Z");
  await nextTick();

  assert.deepEqual(
    keys.map((key) => client.getQueryState(key).isInvalidated),
    [true, true, true, false, false],
  );
});

test("data requests follow the latest successful sync", async (t) => {
  const { meta } = setup(t, "2026-11-20T12:00:00Z");

  assert.equal(lastSyncedRun(), "2026-11-20T12:00:00Z");

  meta.value = {
    lastRunStatus: "error",
    lastRunAt: "2026-11-20T12:15:00Z",
    lastSuccessfulRunAt: null,
  };
  await nextTick();
  assert.equal(lastSyncedRun(), "2026-11-20T12:00:00Z");

  meta.value = successfulSync("2026-11-20T12:30:00Z");
  await nextTick();
  assert.equal(lastSyncedRun(), "2026-11-20T12:30:00Z");
});
