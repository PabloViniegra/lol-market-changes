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

type FieldError = { key: Key; message: string };

const route = useRoute();
const router = useRouter();
const teams = useTeams();

const queryTeam = (key: Key) => {
  const value = route.query[key];

  return (Array.isArray(value) ? value[0] : value)?.trim() ?? "";
};

const names = computed(() => ({ a: queryTeam("a"), b: queryTeam("b") }));
const drafts = ref({ ...names.value });
const fieldError = ref<FieldError | null>(null);
const editing = ref(false);
const changeButton = ref<HTMLButtonElement | null>(null);
const form = ref<HTMLFormElement | null>(null);
const dirty = computed(() => drafts.value.a !== names.value.a || drafts.value.b !== names.value.b);
const duplicateMessage = "Both sides are the same team. Choose a different second team.";

const flipped = ref({ a: false, b: false });

watch(names, (value, previous) => {
  drafts.value = { ...value };
  fieldError.value = null;
  editing.value = false;
  flipped.value = { a: value.a !== previous.a, b: value.b !== previous.b };
});

watch(drafts, () => (fieldError.value = null), { deep: true });

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

function focusField(key: Key) {
  void nextTick(() => form.value?.querySelector<HTMLInputElement>(`#compare-${key}`)?.focus());
}

async function submit() {
  const a = canonical(drafts.value.a);
  const b = canonical(drafts.value.b);
  const missing = !a ? "a" : !b ? "b" : null;

  if (missing) {
    fieldError.value = { key: missing, message: "Pick a team to compare." };
    focusField(missing);

    return;
  }

  if (a.toLowerCase() === b.toLowerCase()) {
    fieldError.value = { key: "b", message: duplicateMessage };
    focusField("b");

    return;
  }

  if (a !== names.value.a || b !== names.value.b) await router.push({ query: { a, b } });

  closeEditor();
}

function swap() {
  drafts.value = { a: drafts.value.b, b: drafts.value.a };
}

function chooseCorrection(key: Key, name: string) {
  drafts.value[key] = name;
  focusField(key);
}

function openEditor() {
  editing.value = true;
  focusField("a");
}

function closeEditor() {
  const restoreFocus = form.value?.contains(document.activeElement);
  drafts.value = { ...names.value };
  fieldError.value = null;
  editing.value = false;

  if (restoreFocus) {
    void nextTick(() => {
      if (changeButton.value?.getClientRects().length) changeButton.value.focus();
      else if (!form.value?.contains(document.activeElement)) focusField("a");
    });
  }
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

const sameTeam = computed(() => {
  const a = teamA.data.value?.team.name ?? canonical(names.value.a);
  const b = teamB.data.value?.team.name ?? canonical(names.value.b);

  return Boolean(a) && a.toLowerCase() === b.toLowerCase();
});

const fields = computed(() =>
  sides.value.map((side) => {
    const isApplied = drafts.value[side.key] === side.name;

    const notFound =
      side.notFound && isApplied
        ? {
            name: side.name,
            suggestion: searchTeams(teams.data.value?.teams ?? [], side.name)[0]?.name,
          }
        : null;

    const hint =
      fieldError.value?.key === side.key
        ? fieldError.value.message
        : side.key === "b" && !dirty.value && sameTeam.value
          ? duplicateMessage
          : null;

    return {
      key: side.key,
      id: `compare-${side.key}`,
      label: side.key === "a" ? "First team" : "Second team",
      notFound,
      hint,
    };
  }),
);

const pending = computed(() => sides.value.some((side) => side.query.isPending.value));
const ready = computed(() => sides.value.some((side) => side.data) && !pending.value);
const both = computed(() => sides.value.every((side) => side.data) && !sameTeam.value);
const compact = computed(() => both.value && !editing.value && !dirty.value);

watch(compact, (value) => {
  if (value && form.value?.contains(document.activeElement)) {
    void nextTick(() => {
      if (changeButton.value?.getClientRects().length) changeButton.value.focus();
    });
  } else if (!value && !editing.value && document.activeElement === changeButton.value) {
    focusField("a");
  }
});

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
    cells: [a, b].map((roster) => {
      const players = roster.filter((entry) => entry.role === role);

      return {
        current: players.filter((entry) => entry.change !== "left"),
        departures: players.filter((entry) => entry.change === "left"),
      };
    }),
  }));
});

function isMainRole(role: string | null) {
  return ROLES.some((item) => item === role);
}

const league = (value: string | null) => LEAGUES.find((item) => item === value) ?? null;

const rumors = useFeed(
  () => ({ team: names.value.a, status: "rumor,confirmed,denied,contradicted,stale", limit: 50 }),
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
  if (!ready.value || sameTeam.value) return "";

  const loaded = sides.value.flatMap((side) => (side.data ? [side.data.team.name] : []));

  return `Showing ${loaded.join(" and ")}.`;
});
</script>

<template>
  <div class="mb-6">
    <h1 class="type-heading-lg">Compare rosters</h1>
    <p class="mt-2 type-body-sm text-text-muted">
      Current rosters by role, with arrivals and departures since {{ seasonLabel }}.
    </p>
  </div>

  <button
    v-if="compact"
    ref="changeButton"
    type="button"
    class="mb-4 min-h-11 rounded-control bg-surface-raised px-4 type-label-md hover:bg-line md:hidden"
    @click="openEditor"
  >
    Change teams
  </button>

  <form
    ref="form"
    class="mb-6 rounded-panel bg-surface p-4 md:mb-10 md:p-6"
    :class="{ 'max-md:hidden': compact }"
    novalidate
    @submit.prevent="submit"
  >
    <div class="flex flex-col gap-4 md:flex-row md:items-start">
      <template v-for="(field, index) in fields" :key="field.key">
        <div v-if="index === 1" class="flex flex-col gap-2 max-md:-my-2 max-md:self-end">
          <span class="invisible type-label-md max-md:hidden" aria-hidden="true">&nbsp;</span>
          <button
            type="button"
            class="min-h-11 rounded-control bg-surface-raised px-3 type-label-md hover:bg-line disabled:opacity-50 md:min-h-10"
            :disabled="drafts.a === drafts.b"
            @click="swap"
          >
            Swap teams
          </button>
        </div>
        <div class="flex min-w-0 flex-1 flex-col gap-2">
          <label :for="field.id" class="type-label-md">{{ field.label }}</label>
          <TeamCombobox
            :id="field.id"
            v-model="drafts[field.key]"
            :teams="teams.data.value?.teams ?? []"
            browse-all
            field-class="h-11 bg-surface-raised md:h-10"
            placeholder="Search a team"
            :aria-invalid="field.notFound || field.hint ? true : undefined"
            :aria-describedby="field.notFound || field.hint ? `${field.id}-hint` : undefined"
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
              @click="chooseCorrection(field.key, field.notFound.suggestion)"
            >
              Did you mean {{ field.notFound.suggestion }}?
            </button>
          </p>
          <p
            v-else-if="field.hint"
            :id="`${field.id}-hint`"
            class="type-body-sm text-text-muted"
            role="alert"
          >
            {{ field.hint }}
          </p>
        </div>
      </template>
      <div class="flex flex-col gap-2">
        <span class="invisible type-label-md max-md:hidden" aria-hidden="true">&nbsp;</span>
        <button
          type="submit"
          class="h-11 rounded-control bg-primary px-4 type-label-md text-on-primary hover:bg-primary-hover md:h-10"
        >
          Compare
        </button>
      </div>
    </div>
    <div v-if="dirty || editing" class="mt-3 flex items-center gap-3">
      <p v-if="dirty" class="type-body-sm text-text-muted" role="status">
        <template v-if="both">Showing the previous comparison. </template>
        Select Compare to apply changes.
      </p>
      <button
        type="button"
        class="min-h-11 rounded-control px-3 type-label-md hover:bg-line md:min-h-10"
        @click="closeEditor"
      >
        Cancel
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

    <template v-else-if="ready && !sameTeam">
      <section aria-labelledby="compare-heading">
        <h2 id="compare-heading" class="sr-only">Rosters</h2>
        <div class="rounded-panel bg-surface px-3 md:px-6">
          <table class="w-full table-fixed text-left" aria-labelledby="compare-heading">
            <thead class="border-b border-line">
              <tr>
                <th scope="col" class="w-16 py-3 pr-2 align-top type-label-md md:w-32">Role</th>
                <th
                  v-for="side in sides"
                  :key="side.key"
                  scope="col"
                  class="py-3 pr-3 align-top last:pr-0"
                >
                  <template v-if="side.data">
                    <RouterLink
                      :to="`/teams/${encodeURIComponent(side.data.team.name)}`"
                      class="flex flex-wrap items-center gap-x-3 gap-y-1 hover:text-primary"
                    >
                      <BoardFlap
                        :key="side.data.team.name"
                        :code="teamCode(side.data.team.name, side.data.team.short)"
                        :logo="side.data.team.logoUrl"
                        placeholder=""
                        :team="null"
                        :flip="flipped[side.key]"
                        class="max-md:hidden"
                      />
                      <span class="wrap-anywhere type-board-player underline">
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
                      <span class="tabular-nums">
                        {{ side.data.arrivals.length }} joined ·
                        {{ side.data.departures.length }} left
                      </span>
                    </span>
                  </template>
                  <template v-else>
                    <span class="wrap-anywhere type-label-md">{{ side.name }}</span>
                    <span class="block type-body-sm text-text-muted">Roster unavailable</span>
                  </template>
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
                  <p
                    v-if="sides[index]?.data && !cell.current.length"
                    class="type-body-sm text-text-muted"
                  >
                    No current player recorded
                  </p>
                  <ul
                    v-for="(players, group) in cell"
                    v-show="players.length"
                    :key="group"
                    :aria-label="group === 'current' ? 'Current players' : 'Departures'"
                    class="flex flex-col gap-2"
                    :class="{ 'mt-2': group === 'departures' }"
                  >
                    <li
                      v-for="entry in players"
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
                        class="min-w-0 wrap-anywhere type-board-player underline hover:text-primary"
                        :class="{ 'text-text-muted': entry.change === 'left' }"
                      >
                        {{ playerHandle(entry.player) }}
                      </RouterLink>
                      <MoveMark v-if="entry.change" :change="entry.change" />
                    </li>
                  </ul>
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
