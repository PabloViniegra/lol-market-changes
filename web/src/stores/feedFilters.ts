import type { FeedStatus, League, MoveKind } from "@lol/shared";
import { defineStore } from "pinia";
import { computed } from "vue";
import { useRoute, useRouter, type LocationQuery } from "vue-router";
import { KINDS, LEAGUES, ROLES, STATUSES } from "../format";

export interface FeedFilters {
  league: League | null;
  role: string | null;
  kind: MoveKind | null;
  team: string | null;
  player: string | null;
  statuses: FeedStatus[];
  from: string | null;
  to: string | null;
}

export const DEFAULT_STATUSES: FeedStatus[] = ["official", "confirmed", "rumor"];

const ISO_DAY = /^\d{4}-\d{2}-\d{2}$/;

function single(query: LocationQuery, key: string): string | null {
  const value = query[key];
  const first = Array.isArray(value) ? value[0] : value;

  return first || null;
}

function pick<T extends string>(value: string | null, allowed: readonly T[]): T | null {
  return allowed.find((item) => item === value) ?? null;
}

function parse(query: LocationQuery): FeedFilters {
  const statusParam = single(query, "status");

  const statuses = statusParam
    ? STATUSES.filter((status) => statusParam.split(",").includes(status))
    : DEFAULT_STATUSES;

  const from = single(query, "from");
  const to = single(query, "to");

  return {
    league: pick(single(query, "league"), LEAGUES),
    role: pick(single(query, "role"), ROLES),
    kind: pick(single(query, "kind"), KINDS),
    team: single(query, "team"),
    player: single(query, "player"),
    statuses: statuses.length > 0 ? statuses : DEFAULT_STATUSES,
    from: from && ISO_DAY.test(from) ? from : null,
    to: to && ISO_DAY.test(to) ? to : null,
  };
}

export function isDefaultStatuses(statuses: FeedStatus[]): boolean {
  return (
    statuses.length === DEFAULT_STATUSES.length &&
    DEFAULT_STATUSES.every((status) => statuses.includes(status))
  );
}

export const useFeedFilters = defineStore("feedFilters", () => {
  const route = useRoute();
  const router = useRouter();
  const filters = computed(() => parse(route.query));

  const apiParams = computed(() => {
    const { statuses, ...rest } = filters.value;

    return { ...rest, view: "market", status: statuses.join(","), limit: 50 };
  });

  const hasNonTeamFilters = computed(() => {
    const { statuses, team: _team, ...rest } = filters.value;

    return !isDefaultStatuses(statuses) || Object.values(rest).some((value) => value !== null);
  });

  const isFiltered = computed(() => hasNonTeamFilters.value || filters.value.team !== null);

  const secondaryFilterCount = computed(() => {
    const { role, kind, team, from, to } = filters.value;

    return [role, kind, team, from, to].filter(Boolean).length;
  });

  function update(patch: Partial<FeedFilters>) {
    const next = { ...filters.value, ...patch };
    const { statuses, ...rest } = next;
    const query: Record<string, string> = {};

    for (const [key, value] of Object.entries(rest)) {
      if (value) query[key] = value;
    }

    if (!isDefaultStatuses(statuses)) query.status = statuses.join(",");

    void router.replace({ query });
  }

  function toggleStatus(status: FeedStatus) {
    const current = filters.value.statuses;

    const statuses = current.includes(status)
      ? current.filter((item) => item !== status)
      : STATUSES.filter((item) => item === status || current.includes(item));

    if (statuses.length > 0) update({ statuses });
  }

  function clear() {
    void router.replace({ query: {} });
  }

  return {
    filters,
    apiParams,
    isFiltered,
    hasNonTeamFilters,
    secondaryFilterCount,
    update,
    toggleStatus,
    clear,
  };
});
