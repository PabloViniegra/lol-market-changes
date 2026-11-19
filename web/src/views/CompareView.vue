<script setup lang="ts">
import type { RosterEntry } from "@lol/shared";
import { computed, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { HttpError } from "../api/client";
import { findTeam, useTeam, useTeams } from "../api/teams";
import BoardFlap from "../components/BoardFlap.vue";
import LeagueTag from "../components/LeagueTag.vue";
import PlayerPortrait from "../components/PlayerPortrait.vue";
import QueryError from "../components/QueryError.vue";
import RoleIcon from "../components/RoleIcon.vue";
import RosterSkeleton from "../components/RosterSkeleton.vue";
import TeamCombobox from "../components/TeamCombobox.vue";
import { LEAGUES, ROLES, playerHandle, teamCode } from "../format";

const route = useRoute();
const router = useRouter();
const teams = useTeams();

const queryTeam = (key: "a" | "b") => {
  const value = route.query[key];

  return (Array.isArray(value) ? value[0] : value)?.trim() ?? "";
};

const nameA = computed(() => queryTeam("a"));
const nameB = computed(() => queryTeam("b"));
const draftA = ref(nameA.value);
const draftB = ref(nameB.value);

watch(nameA, (name) => (draftA.value = name));

watch(nameB, (name) => (draftB.value = name));

const teamA = useTeam(nameA, () => nameA.value !== "");
const teamB = useTeam(nameB, () => nameB.value !== "");

function canonical(draft: string) {
  return findTeam(teams.data.value?.teams, draft)?.name ?? draft.trim();
}

function apply() {
  const query: Record<string, string> = {};
  const a = canonical(draftA.value);
  const b = canonical(draftB.value);

  if (a) query.a = a;

  if (b) query.b = b;

  void router.replace({ query });
}

function swap() {
  [draftA.value, draftB.value] = [draftB.value, draftA.value];
  apply();
}

const sides = computed(() =>
  [
    { key: "a", name: nameA.value, query: teamA },
    { key: "b", name: nameB.value, query: teamB },
  ].map(({ key, name, query }) => ({
    key,
    name,
    query,
    data: query.data.value,
    notFound: query.error.value instanceof HttpError && query.error.value.status === 404,
  })),
);

const ready = computed(() => sides.value.every((side) => side.data));

const rosterRows = computed(() => {
  const [a, b] = sides.value.map((side) => side.data);

  if (!a || !b) return [];

  const roles = new Set<string | null>([
    ...ROLES,
    ...a.roster.map((entry) => entry.role),
    ...b.roster.map((entry) => entry.role),
  ]);

  return Array.from(roles, (role) => ({
    role,
    a: byRole(a.roster, role),
    b: byRole(b.roster, role),
  })).filter((row) => row.a.length || row.b.length);
});

function byRole(roster: RosterEntry[], role: string | null) {
  return roster.filter((entry) => entry.role === role);
}

const league = (value: string | null) => LEAGUES.find((item) => item === value) ?? null;
</script>

<template>
  <div class="mb-6 flex flex-wrap items-baseline justify-between gap-4">
    <h1 class="type-heading-lg">Compare rosters</h1>
    <p class="type-body-sm text-text-muted">Current rosters, with moves since the season start.</p>
  </div>

  <form
    class="mb-10 flex flex-col gap-4 rounded-panel bg-surface p-4 md:flex-row md:items-end md:p-6"
    @submit.prevent="apply"
  >
    <div class="flex min-w-0 flex-1 flex-col gap-2">
      <label for="compare-a" class="type-label-md">First team</label>
      <TeamCombobox
        id="compare-a"
        v-model="draftA"
        :teams="teams.data.value?.teams ?? []"
        placeholder="T1"
        @select="apply"
      />
    </div>
    <button
      type="button"
      class="h-10 rounded-control px-3 type-label-md text-text-muted hover:bg-line hover:text-text"
      aria-label="Swap teams"
      @click="swap"
    >
      ⇄
    </button>
    <div class="flex min-w-0 flex-1 flex-col gap-2">
      <label for="compare-b" class="type-label-md">Second team</label>
      <TeamCombobox
        id="compare-b"
        v-model="draftB"
        :teams="teams.data.value?.teams ?? []"
        placeholder="Gen.G"
        @select="apply"
      />
    </div>
    <button
      type="submit"
      class="h-10 rounded-control bg-primary px-4 type-label-md text-on-primary hover:bg-primary-hover"
    >
      Compare
    </button>
  </form>

  <p v-if="!nameA || !nameB" class="text-text-muted">
    Pick two teams to see their rosters side by side.
  </p>
  <template v-else>
    <template v-for="side in sides" :key="side.key">
      <p v-if="side.notFound" class="mb-4 text-text-muted">
        No tracked team is called “{{ side.name }}”.
      </p>
      <QueryError
        v-else-if="side.query.isError.value"
        :what="side.name"
        class="mb-4"
        @retry="side.query.refetch()"
      />
    </template>

    <div v-if="sides.some((side) => side.query.isPending.value)" aria-busy="true">
      <span class="sr-only">Loading rosters</span>
      <RosterSkeleton />
    </div>

    <section v-else-if="ready" aria-labelledby="compare-heading">
      <h2 id="compare-heading" class="sr-only">Rosters</h2>
      <div class="rounded-panel bg-surface px-3 md:px-6">
        <table class="w-full table-fixed text-left" aria-labelledby="compare-heading">
          <thead class="border-b border-line">
            <tr>
              <th scope="col" class="w-18 py-3 pr-2 align-top type-label-md md:w-32">Role</th>
              <th
                v-for="side in sides"
                :key="side.key"
                scope="col"
                class="py-3 pr-3 align-top type-label-md last:pr-0"
              >
                <template v-if="side.data">
                  <RouterLink
                    :to="`/teams/${encodeURIComponent(side.data.team.name)}`"
                    class="flex flex-wrap items-center gap-2 hover:text-primary"
                  >
                    <BoardFlap
                      :code="teamCode(side.data.team.name, side.data.team.short)"
                      placeholder=""
                      :team="null"
                    />
                    <span class="wrap-anywhere underline">{{ side.data.team.name }}</span>
                  </RouterLink>
                  <span class="mt-2 flex flex-wrap items-center gap-2 type-body-sm text-text-muted">
                    <LeagueTag
                      v-if="league(side.data.team.league)"
                      :league="league(side.data.team.league)!"
                    />
                    {{ side.data.arrivals.length }} in · {{ side.data.departures.length }} out
                  </span>
                </template>
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-line">
            <tr v-if="!rosterRows.length">
              <td colspan="3" class="py-4 text-text-muted">No roster players recorded.</td>
            </tr>
            <tr v-for="row in rosterRows" :key="row.role ?? 0">
              <th scope="row" class="py-3 pr-2 align-top wrap-anywhere type-label-md">
                <span class="flex min-h-8 items-center gap-2">
                  <RoleIcon :role="row.role" class="text-text-muted max-md:hidden" />
                  {{ row.role ?? "Unknown" }}
                </span>
              </th>
              <td
                v-for="(entries, index) in [row.a, row.b]"
                :key="index"
                class="py-3 pr-3 align-top last:pr-0"
              >
                <ul v-if="entries.length" class="flex flex-col gap-2">
                  <li
                    v-for="entry in entries"
                    :key="entry.player"
                    class="flex items-center gap-x-2 md:gap-x-3"
                  >
                    <PlayerPortrait
                      :src="entry.imageUrl"
                      :role="entry.role"
                      :size="32"
                      class="max-md:hidden"
                    />
                    <RouterLink
                      :to="`/players/${encodeURIComponent(entry.player)}`"
                      class="min-w-0 wrap-anywhere underline hover:text-primary"
                    >
                      {{ playerHandle(entry.player) }}
                    </RouterLink>
                    <span
                      v-if="sides[index]?.data?.arrivals.includes(entry.player)"
                      class="type-label-md text-primary"
                    >
                      Joined
                    </span>
                  </li>
                </ul>
                <template v-else>
                  <span class="text-text-muted" aria-hidden="true">—</span>
                  <span class="sr-only">No player recorded</span>
                </template>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </template>
</template>
