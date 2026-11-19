import type { TeamResponse, TeamsResponse } from "@lol/shared";
import { useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { getJson } from "./client";

export function useTeam(name: MaybeRefOrGetter<string>, enabled: MaybeRefOrGetter<boolean> = true) {
  return useQuery({
    enabled,
    queryKey: computed(() => ["team", toValue(name)]),
    queryFn: () => getJson<TeamResponse>(`/api/teams/${encodeURIComponent(toValue(name))}`),
  });
}

export function useTeams(enabled: MaybeRefOrGetter<boolean> = true) {
  return useQuery({
    enabled,
    queryKey: ["teams"],
    queryFn: () => getJson<TeamsResponse>("/api/teams"),
    staleTime: 60 * 60_000,
    refetchInterval: false,
  });
}

export function findTeam(teams: TeamsResponse["teams"] | undefined, query: string) {
  const key = query.trim().toLowerCase();

  return (
    teams?.find((team) => team.name.toLowerCase() === key) ??
    teams?.find((team) => team.short?.toLowerCase() === key)
  );
}

export function searchTeams(teams: TeamsResponse["teams"], query: string) {
  const key = query.trim().toLowerCase();

  const hits = key
    ? teams.filter(
        (team) =>
          team.name.toLowerCase().includes(key) || team.short?.toLowerCase().startsWith(key),
      )
    : teams;

  return hits.toSorted((a, b) => a.league.localeCompare(b.league) || a.name.localeCompare(b.name));
}
