<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useFeed } from "../api/feed";
import { useTeam } from "../api/teams";
import BoardFlap from "../components/BoardFlap.vue";
import FlapSkeleton from "../components/FlapSkeleton.vue";
import LeagueTag from "../components/LeagueTag.vue";
import MoveBoard from "../components/MoveBoard.vue";
import MoveMark from "../components/MoveMark.vue";
import PlayerPortrait from "../components/PlayerPortrait.vue";
import QueryError from "../components/QueryError.vue";
import RoleIcon from "../components/RoleIcon.vue";
import RosterSkeleton from "../components/RosterSkeleton.vue";
import { HttpError } from "../api/client";
import { isDarkLogo, measureLogo } from "../composables/useLogoTone";
import { LEAGUES, ROLES, playerHandle, shortDate, teamCode, tournamentLabel } from "../format";

const props = defineProps<{ name: string }>();

const team = useTeam(() => props.name);

const moves = useFeed(() => ({
  team: props.name,
  status: "official",
  window: "season",
  limit: 50,
}));

const rumors = useFeed(() => ({
  team: props.name,
  status: "rumor,confirmed,denied,contradicted,stale",
  limit: 50,
}));

const notFound = computed(
  () => team.error.value instanceof HttpError && team.error.value.status === 404,
);

const info = computed(() => team.data.value?.team);

const logoFailed = ref(false);

watch(
  () => info.value?.logoUrl,
  () => (logoFailed.value = false),
);

const league = computed(() => LEAGUES.find((item) => item === info.value?.league) ?? null);
const arrivals = computed(() => new Set(team.data.value?.arrivals));
const departures = computed(() => new Set(team.data.value?.departures));
const feedLink = computed(() => `/feed?team=${encodeURIComponent(props.name)}`);

const seasonLabel = computed(() =>
  team.data.value ? shortDate(team.data.value.seasonStart, true) : "",
);

const rosterRows = computed(() => {
  const data = team.data.value;

  if (!data) return [];

  const roles = new Set<string | null>([
    ...ROLES,
    ...data.previousRoster.map((entry) => entry.role),
    ...data.roster.map((entry) => entry.role),
  ]);

  return Array.from(roles, (role) => ({
    role,
    previous: data.previousRoster.filter((entry) => entry.role === role),
    current: data.roster.filter((entry) => entry.role === role),
  })).filter(({ previous, current }) => previous.length || current.length);
});
</script>

<template>
  <div v-if="notFound" class="max-w-prose">
    <h1 class="mb-2 type-heading-lg">Team not found</h1>
    <p class="text-text-muted">
      No tracked team is called “{{ name }}”. Team names follow Leaguepedia, for example “Gen.G”.
    </p>
  </div>
  <QueryError v-else-if="team.isError.value" what="this team" @retry="team.refetch()" />
  <div v-else-if="team.isPending.value" aria-busy="true">
    <span class="sr-only">Loading team</span>
    <div class="mb-10 flex items-center gap-4 md:gap-6" aria-hidden="true">
      <span class="size-20 shrink-0 rounded-flap bg-surface-raised md:size-24" />
      <div class="flex min-w-0 flex-wrap items-center gap-x-4 gap-y-2">
        <span class="h-9 w-full max-w-56 rounded-flap bg-surface-raised" />
        <FlapSkeleton />
        <span class="h-5 w-10 rounded-flap bg-surface-raised" />
      </div>
    </div>
    <span class="mb-4 block h-6 w-24 rounded-flap bg-surface-raised" aria-hidden="true" />
    <RosterSkeleton />
  </div>
  <template v-else-if="team.data.value && info">
    <header class="mb-10 flex items-center gap-4 md:gap-6">
      <span
        v-if="info.logoUrl && !logoFailed"
        class="crest relative flex size-20 shrink-0 items-center justify-center rounded-flap p-3 md:size-24"
        :class="isDarkLogo(info.logoUrl) ? 'bg-text' : 'bg-surface-raised'"
      >
        <img
          :src="info.logoUrl"
          alt=""
          width="72"
          height="72"
          decoding="async"
          referrerpolicy="no-referrer"
          crossorigin="anonymous"
          class="size-full object-contain"
          @load="measureLogo(info.logoUrl, $event.target as HTMLImageElement)"
          @error="logoFailed = true"
        />
      </span>
      <div class="flex min-w-0 flex-wrap items-center gap-x-4 gap-y-2">
        <h1 class="w-full type-heading-lg wrap-anywhere">{{ info.name }}</h1>
        <BoardFlap :code="teamCode(info.name, info.short)" placeholder="" :team="null" />
        <LeagueTag v-if="league" :league="league" />
        <span v-if="info.region" class="type-body-sm text-text-muted">{{ info.region }}</span>
      </div>
    </header>

    <section aria-labelledby="roster-heading">
      <div class="mb-4 flex flex-wrap items-baseline justify-between gap-4">
        <h2 id="roster-heading" class="type-heading-md">Roster</h2>
        <RouterLink
          :to="{ path: '/compare', query: { a: info.name } }"
          class="type-body-sm underline hover:text-primary"
        >
          Compare with another team
        </RouterLink>
      </div>
      <div class="rounded-panel bg-surface px-3 md:px-6">
        <p v-if="!rosterRows.length" class="py-4 text-text-muted">No roster players recorded.</p>
        <table v-else class="w-full table-fixed text-left" aria-labelledby="roster-heading">
          <thead class="border-b border-line">
            <tr>
              <th scope="col" class="w-18 py-3 pr-2 align-top type-label-md md:w-32">Role</th>
              <th scope="col" class="py-3 pr-3 align-top type-label-md">
                Previous roster
                <span class="mt-1 block wrap-anywhere type-body-sm text-text-muted">
                  {{
                    team.data.value.previousTournament
                      ? tournamentLabel(team.data.value.previousTournament)
                      : "No previous roster recorded"
                  }}
                </span>
              </th>
              <th scope="col" class="py-3 align-top type-label-md">
                Current roster
                <span class="mt-1 block type-body-sm text-text-muted">
                  {{
                    team.data.value.roster.length
                      ? `Moves since ${seasonLabel}`
                      : "No current players recorded"
                  }}
                </span>
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
              <td class="py-3 pr-3 align-top">
                <ul v-if="row.previous.length" class="flex flex-col gap-2">
                  <li
                    v-for="entry in row.previous"
                    :key="entry.player"
                    class="flex items-center gap-x-2 md:gap-x-3"
                  >
                    <PlayerPortrait
                      :src="entry.imageUrl"
                      :role="entry.role"
                      :size="32"
                      class="max-md:hidden"
                      :class="{ grayscale: departures.has(entry.player) }"
                    />
                    <RouterLink
                      :to="`/players/${encodeURIComponent(entry.player)}`"
                      class="min-w-0 wrap-anywhere underline hover:text-primary"
                      :class="{ 'text-text-muted': departures.has(entry.player) }"
                    >
                      {{ playerHandle(entry.player) }}
                    </RouterLink>
                    <MoveMark v-if="departures.has(entry.player)" change="left" />
                  </li>
                </ul>
                <template v-else>
                  <span class="text-text-muted" aria-hidden="true">—</span>
                  <span class="sr-only">No player recorded</span>
                </template>
              </td>
              <td class="py-3 align-top">
                <ul v-if="row.current.length" class="flex flex-col gap-2">
                  <li
                    v-for="entry in row.current"
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
                    <MoveMark v-if="arrivals.has(entry.player)" change="joined" />
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

    <section class="mt-12" aria-labelledby="moves-heading">
      <div class="mb-4 flex flex-wrap items-baseline justify-between gap-4">
        <h2 id="moves-heading" class="type-heading-md">Moves since {{ seasonLabel }}</h2>
        <RouterLink :to="feedLink" class="type-body-sm underline hover:text-primary">
          Open in feed
        </RouterLink>
      </div>
      <QueryError v-if="moves.isError.value" what="moves" @retry="moves.refetch()" />
      <MoveBoard
        v-else
        :key="name"
        :items="moves.data.value?.items"
        :pending="moves.isPending.value"
        empty="No official moves for this team since the season started."
      />
    </section>

    <section class="mt-12" aria-labelledby="team-rumors-heading">
      <h2 id="team-rumors-heading" class="mb-4 type-heading-md">Rumors</h2>
      <QueryError v-if="rumors.isError.value" what="rumors" @retry="rumors.refetch()" />
      <MoveBoard
        v-else
        :key="name"
        :items="rumors.data.value?.items"
        :pending="rumors.isPending.value"
        empty="No rumors involve this team."
      />
    </section>
  </template>
</template>

<style scoped>
.crest::after {
  content: "";
  position: absolute;
  inset-inline: 0;
  top: 50%;
  height: 1px;
  background-color: var(--color-line);
}
</style>
