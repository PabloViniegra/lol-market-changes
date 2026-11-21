import assert from "node:assert/strict";
import test from "node:test";
import { syncHealth } from "./syncHealth.ts";

const meta = (lastRunStatus) => ({ lastRunStatus });

test("a failed meta request wins over any cached sync data", () => {
  assert.deepEqual(syncHealth(meta("ok"), true, 5), { state: "unverified", lastSuccessAge: 5 });
  assert.deepEqual(syncHealth(undefined, true, null), {
    state: "unverified",
    lastSuccessAge: null,
  });
});

test("meta still loading is checking, not never synced", () => {
  assert.deepEqual(syncHealth(undefined, false, null), { state: "checking" });
});

test("a failed run reports the age of the last success, even when recent", () => {
  assert.deepEqual(syncHealth(meta("error"), false, 10), { state: "failed", lastSuccessAge: 10 });
  assert.deepEqual(syncHealth(meta("error"), false, null), {
    state: "failed",
    lastSuccessAge: null,
  });
});

test("no successful run yet is never", () => {
  assert.deepEqual(syncHealth(meta("never"), false, null), { state: "never" });
});

test("data turns stale only after 60 minutes", () => {
  assert.deepEqual(syncHealth(meta("ok"), false, 60), { state: "fresh", age: 60 });
  assert.deepEqual(syncHealth(meta("ok"), false, 61), { state: "stale", age: 61 });
});
