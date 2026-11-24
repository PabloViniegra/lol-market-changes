import type { TeamResponse, TeamsResponse } from "@lol/shared";
import { useQuery, useQueryClient, type QueryFunctionContext } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";
import { getData } from "./client";
import { queryKeys } from "./keys";

const TEAM_CATALOG_CACHE_TIME = 60 * 60_000;

function fetchTeam({
  queryKey: [, , name],
  signal,
}: QueryFunctionContext<ReturnType<typeof queryKeys.team>>) {
  return getData<TeamResponse>(`/api/teams/${encodeURIComponent(name)}`, {}, signal);
}

export function useTeam(name: MaybeRefOrGetter<string>, enabled: MaybeRefOrGetter<boolean> = true) {
  return useQuery({
    enabled,
    queryKey: computed(() => queryKeys.team(toValue(name))),
    queryFn: fetchTeam,
  });
}

export function usePrefetchTeam() {
  const queryClient = useQueryClient();

  return (name: string) =>
    void queryClient.prefetchQuery({ queryKey: queryKeys.team(name), queryFn: fetchTeam });
}

export function useTeams(enabled: MaybeRefOrGetter<boolean> = true) {
  return useQuery({
    enabled,
    queryKey: queryKeys.teams,
    queryFn: ({ signal }) => getData<TeamsResponse>("/api/teams", {}, signal),
    staleTime: TEAM_CATALOG_CACHE_TIME,
    gcTime: TEAM_CATALOG_CACHE_TIME,
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
