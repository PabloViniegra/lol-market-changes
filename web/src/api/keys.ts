import type { FlowsResponse, StatsWindow } from "@lol/shared";
import type { QueryParams } from "./client";

// The feed and team endpoints match team and player names case-insensitively, so case variants
// share one cache entry. Player pages match exactly and keep the name as typed.
const CASE_INSENSITIVE_PARAMS = new Set(["team", "player"]);

function normalizeParams(params: QueryParams): QueryParams {
  const normalized: QueryParams = {};

  for (const [key, value] of Object.entries(params)) {
    if (value === null || value === undefined || value === "") continue;

    normalized[key] = CASE_INSENSITIVE_PARAMS.has(key) ? String(value).toLowerCase() : value;
  }

  return normalized;
}

export const queryKeys = {
  meta: ["meta"] as const,
  teams: ["teams"] as const,
  team: (name: string) => ["team", name.toLowerCase()] as const,
  player: (name: string) => ["player", name] as const,
  stats: (window: StatsWindow) => ["stats", window] as const,
  flows: (level: FlowsResponse["level"], window: StatsWindow) => ["flows", level, window] as const,
  feed: {
    list: (params: QueryParams) => ["feed", "list", normalizeParams(params)] as const,
    pages: (params: QueryParams) => ["feed", "pages", normalizeParams(params)] as const,
  },
};
