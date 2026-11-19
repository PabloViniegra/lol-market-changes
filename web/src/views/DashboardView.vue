<script setup lang="ts">
import type { FlowsResponse } from "@lol/shared";
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useFeed } from "../api/feed";
import { useFlows, useStats } from "../api/stats";
import BoardLegend from "../components/BoardLegend.vue";
import FilterChip from "../components/FilterChip.vue";
import FlowsChart from "../components/FlowsChart.vue";
import LeagueBars from "../components/LeagueBars.vue";
import MoveBoard from "../components/MoveBoard.vue";
import QueryError from "../components/QueryError.vue";
import { shortDate } from "../format";

const route = useRoute();
const router = useRouter();
const allMoves = computed(() => route.query.moves === "all");

const latest = useFeed(() => ({
  view: "market",
  kind: allMoves.value ? null : "transfer",
  limit: 12,
}));

const rumors = useFeed({ status: "rumor", limit: 6 });
const stats = useStats("season");
const flowLevel = computed(() => (route.query.flow === "team" ? "team" : "league"));
const flows = useFlows(flowLevel, "season");

function setFlowLevel(level: FlowsResponse["level"]) {
  void router.replace({
    query: { ...route.query, flow: level === "team" ? level : undefined },
    hash: route.hash,
  });
}

function setAllMoves(all: boolean) {
  void router.replace({
    query: { ...route.query, moves: all ? "all" : undefined },
    hash: route.hash,
  });
}

const teamToTeam = computed(
  () => flows.data.value?.links.reduce((sum, link) => sum + link.value, 0) ?? 0,
);
</script>

<template>
  <div class="flex flex-col gap-12 xl:flex-row xl:gap-6">
    <section class="min-w-0 flex-1" aria-labelledby="latest-heading">
      <div class="mb-4 flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <h1 id="latest-heading" class="type-heading-lg">Latest moves</h1>
        <div class="mr-auto flex gap-2" role="group" aria-label="Moves shown">
          <FilterChip :active="!allMoves" @click="setAllMoves(false)">Transfers</FilterChip>
          <FilterChip :active="allMoves" @click="setAllMoves(true)">All moves</FilterChip>
        </div>
        <RouterLink
          :to="allMoves ? '/feed' : '/feed?kind=transfer'"
          class="inline-flex h-11 items-center rounded-control bg-primary px-4 type-label-md text-on-primary hover:bg-primary-hover md:h-10"
        >
          {{ allMoves ? "See all moves" : "See all transfers" }}
        </RouterLink>
        <BoardLegend />
      </div>
      <section
        v-if="rumors.isError.value || rumors.data.value?.items.length"
        class="relative mb-4 border-y border-line py-2"
        aria-label="Open rumors"
      >
        <details v-if="rumors.data.value?.items.length" class="group">
          <summary
            class="mr-28 inline-flex min-h-11 cursor-pointer items-center gap-1 rounded-control type-label-md hover:text-primary md:min-h-8"
          >
            Rumors to watch
            <span class="group-open:rotate-90" aria-hidden="true">▸</span>
          </summary>
          <div class="mt-2 mb-2">
            <MoveBoard :items="rumors.data.value.items" :pending="false" />
          </div>
        </details>
        <p v-else class="flex min-h-11 items-center pr-28 type-body-sm text-text-muted md:min-h-8">
          Open rumors
        </p>
        <RouterLink
          to="/rumors"
          class="absolute top-2 right-0 inline-flex min-h-11 items-center rounded-control type-body-sm underline hover:text-primary md:min-h-8"
        >
          Rumor tracker
        </RouterLink>
        <QueryError v-if="rumors.isError.value" what="open rumors" @retry="rumors.refetch()" />
      </section>
      <div class="relative">
        <a
          href="#window-heading"
          class="sr-only focus:not-sr-only focus:absolute focus:z-10 focus:rounded-control focus:bg-surface-raised focus:p-3"
          >Skip to window summary</a
        >
        <a
          href="#flows-heading"
          class="sr-only focus:not-sr-only focus:absolute focus:z-10 focus:rounded-control focus:bg-surface-raised focus:p-3"
          >Skip to player flows</a
        >
      </div>
      <QueryError v-if="latest.isError.value" what="moves" @retry="latest.refetch()" />
      <MoveBoard
        v-else
        :items="latest.data.value?.items"
        :pending="latest.isPending.value"
        :rows="12"
        :empty="allMoves ? 'No moves to show yet.' : 'No transfers to show yet.'"
      />
    </section>

    <aside class="flex flex-col gap-4 xl:w-72" aria-labelledby="window-heading">
      <h2
        id="window-heading"
        tabindex="-1"
        class="type-heading-md xl:flex xl:min-h-10 xl:items-center"
      >
        This window
      </h2>
      <QueryError v-if="stats.isError.value" what="stats" @retry="stats.refetch()" />
      <div v-else-if="stats.data.value" class="rounded-panel bg-surface p-6">
        <p class="mb-4 flex flex-wrap items-baseline gap-x-2 type-body-sm text-text-muted">
          <strong class="type-heading-lg text-text tabular-nums">{{
            stats.data.value.totalMoves
          }}</strong>
          <span>moves since {{ shortDate(stats.data.value.since) }}</span>
        </p>
        <LeagueBars :stats="stats.data.value" />
        <dl class="my-4 flex flex-col gap-3 border-t border-line pt-4 type-body-sm">
          <div class="flex items-baseline justify-between gap-4">
            <dt class="text-text-muted">Released and unsigned</dt>
            <dd class="shrink-0 type-data-tabular">{{ stats.data.value.freeAgents }}</dd>
          </div>
          <div
            v-if="stats.data.value.rumorConfirmationRate !== null"
            class="flex items-baseline justify-between gap-4"
          >
            <dt class="text-text-muted">Share of resolved rumors confirmed</dt>
            <dd class="shrink-0 type-data-tabular">
              {{ Math.round(stats.data.value.rumorConfirmationRate * 100) }}%
            </dd>
          </div>
        </dl>
        <div v-if="stats.data.value.topTeamsByChurn.length" class="border-t border-line pt-4">
          <div class="mb-1 flex justify-between gap-4">
            <h3 class="type-label-md">Busiest teams</h3>
            <span class="type-body-sm text-text-muted">Joined / left</span>
          </div>
          <ul class="flex flex-col type-body-sm">
            <li
              v-for="team in stats.data.value.topTeamsByChurn.slice(0, 5)"
              :key="team.team"
              class="flex items-center justify-between gap-4"
            >
              <RouterLink
                :to="`/teams/${encodeURIComponent(team.team)}`"
                class="min-w-0 flex-1 truncate py-2.5 underline hover:text-primary md:py-1"
              >
                {{ team.team }}
              </RouterLink>
              <span class="whitespace-nowrap text-text-muted tabular-nums">
                +{{ team.arrivals }} / −{{ team.departures }}
              </span>
            </li>
          </ul>
        </div>
      </div>
      <div v-else-if="stats.isPending.value" class="rounded-panel bg-surface p-6" aria-busy="true">
        <span class="sr-only">Loading stats</span>
        <div aria-hidden="true">
          <div class="mb-4 flex h-10 items-baseline gap-2">
            <span class="h-10 w-12 rounded-flap bg-surface-raised" />
            <span class="h-3 w-28 rounded-flap bg-surface-raised" />
          </div>
          <ul class="flex flex-col gap-2">
            <li
              v-for="width in ['w-full', 'w-3/4', 'w-1/2', 'w-2/3']"
              :key="width"
              class="flex items-center gap-3"
            >
              <span class="h-3 w-9 rounded-flap bg-surface-raised" />
              <span class="h-3 flex-1">
                <span class="block h-full rounded-flap bg-surface-raised" :class="width" />
              </span>
              <span class="h-3 w-8 rounded-flap bg-surface-raised" />
            </li>
          </ul>
        </div>
        <div class="my-4 flex flex-col gap-3 border-t border-line pt-4" aria-hidden="true">
          <div v-for="n in 2" :key="n" class="flex justify-between gap-4">
            <span class="h-3 w-28 rounded-flap bg-surface-raised" />
            <span class="h-3 w-12 rounded-flap bg-surface-raised" />
          </div>
        </div>
        <div class="border-t border-line pt-4" aria-hidden="true">
          <div class="mb-3 flex justify-between gap-4">
            <span class="h-3 w-24 rounded-flap bg-surface-raised" />
            <span class="h-3 w-20 rounded-flap bg-surface-raised" />
          </div>
          <ul class="flex flex-col gap-4 py-1">
            <li
              v-for="width in ['w-32', 'w-24', 'w-36', 'w-28', 'w-20']"
              :key="width"
              class="flex justify-between gap-4"
            >
              <span class="h-3 rounded-flap bg-surface-raised" :class="width" />
              <span class="h-3 w-12 rounded-flap bg-surface-raised" />
            </li>
          </ul>
        </div>
      </div>
    </aside>
  </div>

  <section class="mt-12" aria-labelledby="flows-heading">
    <div class="mb-4 flex flex-wrap items-center justify-between gap-4">
      <h2 id="flows-heading" tabindex="-1" class="type-heading-md">Where players went</h2>
      <div class="flex gap-2" role="group" aria-label="Flow level">
        <FilterChip :active="flowLevel === 'league'" @click="setFlowLevel('league')"
          >By league</FilterChip
        >
        <FilterChip :active="flowLevel === 'team'" @click="setFlowLevel('team')"
          >By team</FilterChip
        >
      </div>
    </div>
    <QueryError v-if="flows.isError.value" what="flows" @retry="flows.refetch()" />
    <div v-else class="rounded-panel bg-surface p-4 md:p-6">
      <div v-if="flows.isPending.value" aria-busy="true">
        <span class="sr-only">Loading flows</span>
        <div class="mb-6 flex max-w-prose flex-col gap-2" aria-hidden="true">
          <span class="h-3 w-full rounded-flap bg-surface-raised" />
          <span class="h-3 w-2/3 rounded-flap bg-surface-raised" />
        </div>
        <div class="flex h-80 justify-between gap-6" aria-hidden="true">
          <div class="flex w-3 flex-col gap-3">
            <span
              v-for="(grow, i) in ['grow-3', 'grow-2', 'grow', 'grow-2']"
              :key="i"
              class="rounded-flap bg-surface-raised"
              :class="grow"
            />
          </div>
          <div class="flex w-3 flex-col gap-3">
            <span
              v-for="(grow, i) in ['grow-2', 'grow-3', 'grow', 'grow']"
              :key="i"
              class="rounded-flap bg-surface-raised"
              :class="grow"
            />
          </div>
        </div>
      </div>
      <p v-else-if="!flows.data.value?.links.length" class="text-text-muted">
        No team-to-team moves this window yet.
      </p>
      <div
        v-else
        :aria-busy="flows.isPlaceholderData.value"
        :class="{ 'opacity-60': flows.isPlaceholderData.value }"
      >
        <p class="mb-4 max-w-prose type-body-sm text-text-muted">
          {{ teamToTeam }} {{ teamToTeam === 1 ? "move" : "moves" }}
          <template v-if="stats.data.value">of {{ stats.data.value.totalMoves }}</template>
          went straight from one team to another. Releases, retirements and free agent signings are
          not shown.
          <template v-if="flows.data.value.level === 'league'">
            Other covers teams outside LCK, LPL, LEC and LCS.
          </template>
        </p>
        <FlowsChart :links="flows.data.value.links" />
      </div>
    </div>
  </section>
</template>
