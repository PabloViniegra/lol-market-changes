<script setup lang="ts">
import type { RosterEntry, TeamResponse } from "@lol/shared";
import { computed, nextTick, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { HttpError } from "../api/client";
import { useFeed } from "../api/feed";
import { findTeam, searchTeams, useTeam, useTeams } from "../api/teams";
import BoardFlap from "../components/BoardFlap.vue";
import LeagueTag from "../components/LeagueTag.vue";
import MoveBoard from "../components/MoveBoard.vue";
import MoveMark from "../components/MoveMark.vue";
import PlayerPortrait from "../components/PlayerPortrait.vue";
import QueryError from "../components/QueryError.vue";
import RoleIcon from "../components/RoleIcon.vue";
import RosterSkeleton from "../components/RosterSkeleton.vue";
import TeamCombobox from "../components/TeamCombobox.vue";
import { LEAGUES, ROLES, playerHandle, shortDate, teamCode } from "../format";

type Key = "a" | "b";

const route = useRoute();
const router = useRouter();
const teams = useTeams();

const queryTeam = (key: Key) => {
  const value = route.query[key];

  return (Array.isArray(value) ? value[0] : value)?.trim() ?? "";
};

const names = computed(() => ({ a: queryTeam("a"), b: queryTeam("b") }));
const drafts = ref({ a: names.value.a, b: names.value.b });
const missing = ref<Key | null>(null);

watch(names, (value) => (drafts.value = { ...value }));

watch(
  drafts,
  (value) => {
    if (missing.value && value[missing.value].trim()) missing.value = null;
  },
  { deep: true },
);

const teamA = useTeam(
  () => names.value.a,
  () => names.value.a !== "",
);

const teamB = useTeam(
  () => names.value.b,
  () => names.value.b !== "",
);

function canonical(draft: string) {
  return findTeam(teams.data.value?.teams, draft)?.name ?? draft.trim();
}

function apply() {
  const query: Record<string, string> = {};
  const a = canonical(drafts.value.a);
  const b = canonical(drafts.value.b);

  if (a) query.a = a;

  if (b) query.b = b;

  void router.replace({ query });
}

function submit() {
  missing.value = (["a", "b"] as const).find((key) => !drafts.value[key].trim()) ?? null;

  if (missing.value) {
    const id = `compare-${missing.value}`;

    void nextTick(() => document.getElementById(id)?.focus());

    return;
  }

  apply();
}

function swap() {
  drafts.value = { a: drafts.value.b, b: drafts.value.a };
  apply();
}

function useSuggestion(key: Key, name: string) {
  drafts.value[key] = name;
  apply();
}

const sides = computed(() =>
  (
    [
      { key: "a", query: teamA },
      { key: "b", query: teamB },
    ] as const
  ).map(({ key, query }) => ({
    key,
    name: names.value[key],
    query,
    data: query.data.value,
    notFound: query.error.value instanceof HttpError && query.error.value.status === 404,
  })),
);

const sameTeam = computed(() => names.value.a !== "" && names.value.a === names.value.b);

const fields = computed(() =>
  sides.value.map((side) => {
    const notFound = side.notFound
      ? {
          name: side.name,
          suggestion: searchTeams(teams.data.value?.teams ?? [], side.name)[0]?.name,
        }
      : null;

    const hint =
      missing.value === side.key
        ? "Pick a team to compare."
        : !notFound && side.key === "b" && sameTeam.value
          ? "Both sides are the same team. Pick another one to compare."
          : null;

    return {
      key: side.key,
      id: `compare-${side.key}`,
      label: side.key === "a" ? "First team" : "Second team",
      notFound,
      hint,
      invalid: Boolean(notFound) || missing.value === side.key,
    };
  }),
);

const pending = computed(() => sides.value.some((side) => side.query.isPending.value));
const ready = computed(() => sides.value.some((side) => side.data) && !pending.value);
const both = computed(() => sides.value.every((side) => side.data) && !sameTeam.value);

const seasonLabel = computed(() => {
  const start = sides.value.find((side) => side.data)?.data?.seasonStart;

  return start ? shortDate(start, true) : "the season start";
});

type Entry = RosterEntry & { change: "joined" | "left" | null };

function entries(data: TeamResponse | undefined): Entry[] {
  if (!data) return [];

  const arrivals = new Set(data.arrivals);
  const departures = new Set(data.departures);

  const current: Entry[] = data.roster.map((entry) => ({
    ...entry,
    change: arrivals.has(entry.player) ? "joined" : null,
  }));

  const left: Entry[] = data.previousRoster.flatMap((entry) =>
    departures.has(entry.player) ? [{ ...entry, change: "left" }] : [],
  );

  return [...current, ...left];
}

const rosterRows = computed(() => {
  const [a = [], b = []] = sides.value.map((side) => entries(side.data));
  const extra = [...a, ...b].flatMap((entry) => (isMainRole(entry.role) ? [] : [entry.role]));

  return [...new Set<string | null>([...ROLES, ...extra])].map((role) => ({
    role,
    cells: [byRole(a, role), byRole(b, role)],
  }));
});

function isMainRole(role: string | null) {
  return ROLES.some((item) => item === role);
}

function byRole(roster: Entry[], role: string | null) {
  return roster.filter((entry) => entry.role === role);
}

const league = (value: string | null) => LEAGUES.find((item) => item === value) ?? null;

const rumors = useFeed(
  () => ({ team: names.value.a, status: "rumor,confirmed,denied,stale", limit: 50 }),
  both,
);

const linkingRumors = computed(() => {
  const [a, b] = sides.value.map((side) => side.data?.team.name.toLowerCase());

  return rumors.data.value?.items.filter((item) => {
    const ends = [item.fromTeam?.toLowerCase(), item.toTeam?.toLowerCase()];

    return ends.includes(a) && ends.includes(b);
  });
});

const summary = computed(() => {
  if (!ready.value) return "";

  const loaded = sides.value.flatMap((side) => (side.data ? [side.data.team.name] : []));

  return `Showing ${loaded.join(" and ")}.`;
});
</script>

<template>
  <div class="mb-6">
    <h1 class="type-heading-lg">Compare rosters</h1>
    <p class="mt-2 type-body-sm text-text-muted">
      Current rosters by role. Joined and Left mark moves since {{ seasonLabel }}.
    </p>
  </div>

  <form
    class="mb-10 flex flex-col gap-4 rounded-panel bg-surface p-4 md:flex-row md:items-start md:p-6"
    novalidate
    @submit.prevent="submit"
  >
    <template v-for="(field, index) in fields" :key="field.key">
      <div v-if="index === 1" class="flex flex-col gap-2 max-md:-my-2 max-md:self-end">
        <span class="invisible type-label-md max-md:hidden" aria-hidden="true">&nbsp;</span>
        <button
          type="button"
          class="flex h-8 items-center gap-2 rounded-control px-3 type-label-md text-text-muted hover:bg-line hover:text-text md:h-10"
          @click="swap"
        >
          <span aria-hidden="true">⇄</span>
          <span class="md:sr-only">Swap teams</span>
        </button>
      </div>
      <div class="flex min-w-0 flex-1 flex-col gap-2">
        <label :for="field.id" class="type-label-md">{{ field.label }}</label>
        <TeamCombobox
          :id="field.id"
          v-model="drafts[field.key]"
          :teams="teams.data.value?.teams ?? []"
          placeholder="Search a team"
          :aria-invalid="field.invalid ? true : undefined"
          :aria-describedby="field.notFound || field.hint ? `${field.id}-hint` : undefined"
          @select="apply"
        />
        <p
          v-if="field.notFound"
          :id="`${field.id}-hint`"
          class="type-body-sm text-text-muted"
          role="alert"
        >
          No tracked team is called “{{ field.notFound.name }}”.
          <button
            v-if="field.notFound.suggestion"
            type="button"
            class="underline hover:text-primary"
            @click="useSuggestion(field.key, field.notFound.suggestion)"
          >
            Did you mean {{ field.notFound.suggestion }}?
          </button>
        </p>
        <p v-else-if="field.hint" :id="`${field.id}-hint`" class="type-body-sm text-text-muted">
          {{ field.hint }}
        </p>
      </div>
    </template>
    <div class="flex flex-col gap-2">
      <span class="invisible type-label-md max-md:hidden" aria-hidden="true">&nbsp;</span>
      <button
        type="submit"
        class="h-10 rounded-control bg-primary px-4 type-label-md text-on-primary hover:bg-primary-hover"
      >
        Compare
      </button>
    </div>
  </form>

  <p class="sr-only" aria-live="polite">{{ summary }}</p>

  <p v-if="!names.a || !names.b" class="text-text-muted">
    Pick two teams to see their rosters side by side.
  </p>
  <template v-else>
    <template v-for="side in sides" :key="side.key">
      <QueryError
        v-if="side.query.isError.value && !side.notFound"
        :what="side.name"
        class="mb-4"
        @retry="side.query.refetch()"
      />
    </template>

    <div v-if="pending" aria-busy="true">
      <span class="sr-only">Loading rosters</span>
      <RosterSkeleton />
    </div>

    <template v-else-if="ready">
      <section aria-labelledby="compare-heading">
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
                  class="py-3 pr-3 align-top last:pr-0"
                >
                  <template v-if="side.data">
                    <RouterLink
                      :to="`/teams/${encodeURIComponent(side.data.team.name)}`"
                      class="flex flex-wrap items-center gap-x-3 gap-y-2 hover:text-primary"
                    >
                      <BoardFlap
                        :code="teamCode(side.data.team.name, side.data.team.short)"
                        :logo="side.data.team.logoUrl"
                        placeholder=""
                        :team="null"
                      />
                      <span class="wrap-anywhere type-body-md underline">
                        {{ side.data.team.name }}
                      </span>
                    </RouterLink>
                    <span
                      class="mt-2 flex flex-wrap items-center gap-2 type-body-sm text-text-muted"
                    >
                      <LeagueTag
                        v-if="league(side.data.team.league)"
                        :league="league(side.data.team.league)!"
                      />
                      {{ side.data.arrivals.length }} joined ·
                      {{ side.data.departures.length }} left
                    </span>
                  </template>
                  <span v-else class="text-text-muted">—</span>
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-line">
              <tr v-for="row in rosterRows" :key="row.role ?? 0">
                <th scope="row" class="py-3 pr-2 align-top wrap-anywhere type-label-md">
                  <span class="flex min-h-8 items-center gap-2">
                    <RoleIcon :role="row.role" class="text-text-muted max-md:hidden" />
                    {{ row.role ?? "Unknown" }}
                  </span>
                </th>
                <td
                  v-for="(cell, index) in row.cells"
                  :key="index"
                  class="py-3 pr-3 align-top last:pr-0"
                >
                  <ul v-if="cell.length" class="flex flex-col gap-2">
                    <li
                      v-for="entry in cell"
                      :key="`${entry.player}-${entry.change}`"
                      class="flex flex-wrap items-center gap-x-2 md:gap-x-3"
                    >
                      <PlayerPortrait
                        :src="entry.imageUrl"
                        :role="entry.role"
                        :size="32"
                        class="max-md:hidden"
                        :class="{ grayscale: entry.change === 'left' }"
                      />
                      <RouterLink
                        :to="`/players/${encodeURIComponent(entry.player)}`"
                        class="min-w-0 wrap-anywhere underline hover:text-primary"
                        :class="{ 'text-text-muted': entry.change === 'left' }"
                      >
                        {{ playerHandle(entry.player) }}
                      </RouterLink>
                      <MoveMark v-if="entry.change" :change="entry.change" />
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

      <section v-if="both" class="mt-12" aria-labelledby="compare-rumors-heading">
        <h2 id="compare-rumors-heading" class="mb-4 type-heading-md">Rumors between them</h2>
        <QueryError v-if="rumors.isError.value" what="rumors" @retry="rumors.refetch()" />
        <MoveBoard
          v-else
          :items="linkingRumors"
          :pending="rumors.isPending.value"
          :rows="2"
          empty="No rumors link these two teams."
        />
      </section>
    </template>
  </template>
</template>
