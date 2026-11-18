<script setup lang="ts">
import { computed } from "vue";
import { useFeed } from "../api/feed";
import { useTeam } from "../api/teams";
import BoardFlap from "../components/BoardFlap.vue";
import LeagueTag from "../components/LeagueTag.vue";
import MoveBoard from "../components/MoveBoard.vue";
import QueryError from "../components/QueryError.vue";
import { HttpError } from "../api/client";
import { LEAGUES, playerHandle, shortDate, teamCode, tournamentLabel } from "../format";

const props = defineProps<{ name: string }>();

const team = useTeam(() => props.name);

const moves = useFeed(
  () => ({ team: props.name, status: "official", from: team.data.value?.seasonStart, limit: 50 }),
  () => team.data.value !== undefined,
);

const rumors = useFeed(() => ({
  team: props.name,
  status: "rumor,confirmed,denied,stale",
  limit: 50,
}));

const notFound = computed(
  () => team.error.value instanceof HttpError && team.error.value.status === 404,
);

const info = computed(() => team.data.value?.team);
const league = computed(() => LEAGUES.find((item) => item === info.value?.league) ?? null);
const arrivals = computed(() => new Set(team.data.value?.arrivals));
const departures = computed(() => new Set(team.data.value?.departures));
const feedLink = computed(() => `/feed?team=${encodeURIComponent(props.name)}`);

const seasonLabel = computed(() =>
  team.data.value ? shortDate(team.data.value.seasonStart, true) : "",
);
</script>

<template>
  <div v-if="notFound" class="max-w-prose">
    <h1 class="mb-2 type-heading-lg">Team not found</h1>
    <p class="text-text-muted">
      No tracked team is called “{{ name }}”. Team names follow Leaguepedia, for example “Gen.G”.
    </p>
  </div>
  <QueryError v-else-if="team.isError.value" what="this team" @retry="team.refetch()" />
  <div v-else-if="team.isPending.value" class="h-40" aria-busy="true" />
  <template v-else-if="team.data.value && info">
    <header class="mb-10 flex flex-wrap items-center gap-4">
      <BoardFlap :code="teamCode(info.name, info.short)" placeholder="" :team="null" />
      <h1 class="type-heading-lg">{{ info.name }}</h1>
      <LeagueTag v-if="league" :league="league" />
      <span v-if="info.region" class="type-body-sm text-text-muted">{{ info.region }}</span>
    </header>

    <section aria-labelledby="roster-heading">
      <h2 id="roster-heading" class="mb-4 type-heading-md">Roster</h2>
      <div class="grid gap-6 md:grid-cols-2">
        <div class="rounded-panel bg-surface p-6">
          <h3 class="type-label-md">Previous season</h3>
          <p class="mb-4 type-body-sm text-text-muted">
            {{
              team.data.value.previousTournament
                ? tournamentLabel(team.data.value.previousTournament)
                : "No tournament roster before this season"
            }}
          </p>
          <ul class="flex flex-col divide-y divide-line">
            <li
              v-for="entry in team.data.value.previousRoster"
              :key="entry.player"
              class="flex items-baseline justify-between gap-4 py-2"
            >
              <RouterLink
                :to="`/players/${encodeURIComponent(entry.player)}`"
                class="underline hover:text-primary"
                :class="departures.has(entry.player) ? 'text-text-muted' : ''"
              >
                {{ playerHandle(entry.player) }}
              </RouterLink>
              <span class="flex gap-3 type-body-sm text-text-muted">
                {{ entry.role }}
                <span v-if="departures.has(entry.player)" class="type-label-md">Left</span>
              </span>
            </li>
          </ul>
        </div>

        <div class="rounded-panel bg-surface p-6">
          <h3 class="type-label-md">Now</h3>
          <p class="mb-4 type-body-sm text-text-muted">With every move since {{ seasonLabel }}</p>
          <p v-if="!team.data.value.roster.length" class="text-text-muted">
            No players under contract yet.
          </p>
          <ul v-else class="flex flex-col divide-y divide-line">
            <li
              v-for="entry in team.data.value.roster"
              :key="entry.player"
              class="flex items-baseline justify-between gap-4 py-2"
            >
              <RouterLink
                :to="`/players/${encodeURIComponent(entry.player)}`"
                class="underline hover:text-primary"
              >
                {{ playerHandle(entry.player) }}
              </RouterLink>
              <span class="flex gap-3 type-body-sm text-text-muted">
                {{ entry.role }}
                <span v-if="arrivals.has(entry.player)" class="type-label-md text-primary"
                  >Joined</span
                >
              </span>
            </li>
          </ul>
        </div>
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
