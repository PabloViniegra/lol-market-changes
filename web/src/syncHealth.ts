import type { MetaResponse } from "@lol/shared";

const STALE_AFTER_MINUTES = 60;

export type SyncHealth =
  | { state: "checking" }
  | { state: "never" }
  | { state: "fresh"; age: number }
  | { state: "stale"; age: number }
  | { state: "failed"; lastSuccessAge: number | null }
  | { state: "unverified"; lastSuccessAge: number | null };

export function syncHealth(
  meta: MetaResponse | undefined,
  metaUnavailable: boolean,
  lastSuccessAge: number | null,
): SyncHealth {
  if (metaUnavailable) return { state: "unverified", lastSuccessAge };

  if (!meta) return { state: "checking" };

  if (meta.lastRunStatus === "error") return { state: "failed", lastSuccessAge };

  if (lastSuccessAge === null) return { state: "never" };

  return lastSuccessAge > STALE_AFTER_MINUTES
    ? { state: "stale", age: lastSuccessAge }
    : { state: "fresh", age: lastSuccessAge };
}
