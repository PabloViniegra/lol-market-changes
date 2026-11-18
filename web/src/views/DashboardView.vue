<script setup lang="ts">
import type { FlowsResponse } from "@lol/shared";
import { computed, defineAsyncComponent, ref } from "vue";
import { useFeed } from "../api/feed";
import { useFlows, useStats } from "../api/stats";
import BoardLegend from "../components/BoardLegend.vue";
import FilterChip from "../components/FilterChip.vue";
import LeagueBars from "../components/LeagueBars.vue";
import MoveBoard from "../components/MoveBoard.vue";
import QueryError from "../components/QueryError.vue";
import { shortDate } from "../format";

const FlowsChart = defineAsyncComponent(() => import("../components/FlowsChart.vue"));

const latest = useFeed({ limit: 12 });
const rumors = useFeed({ status: "rumor", limit: 6 });
const stats = useStats("season");
const flowLevel = ref<FlowsResponse["level"]>("league");
const flows = useFlows(flowLevel, "season");

const teamToTeam = computed(
  () => flows.data.value?.links.reduce((sum, link) => sum + link.value, 0) ?? 0,
);
</script>

<template>
  <h1 class="sr-only">Dashboard</h1>

  <div class="flex flex-col gap-12 lg:flex-row lg:gap-6">
    <section class="min-w-0 flex-1" aria-labelledby="latest-heading">
      <div class="mb-2 flex flex-wrap items-baseline justify-between gap-x-4">
        <h2 id="latest-heading" class="type-heading-md">Latest moves</h2>
        <BoardLegend />
      </div>
      <QueryError v-if="latest.isError.value" what="moves" @retry="latest.refetch()" />
      <MoveBoard
        v-else
        :items="latest.data.value?.items"
        :pending="latest.isPending.value"
        empty="No moves yet this window. New moves show up here within 30 minutes of Leaguepedia."
      />
      <RouterLink
        to="/feed"
        class="mt-4 inline-flex h-10 items-center rounded-control bg-primary px-4 type-label-md text-on-primary hover:bg-primary-hover"
      >
        See all moves
      </RouterLink>
    </section>

    <aside class="flex flex-col gap-8 lg:w-80" aria-labelledby="window-heading">
      <h2 id="window-heading" class="type-heading-md">This window</h2>
      <QueryError v-if="stats.isError.value" what="stats" @retry="stats.refetch()" />
      <template v-else-if="stats.data.value">
        <div class="rounded-panel bg-surface p-6">
          <p class="mb-4 type-body-sm text-text-muted">
            {{ stats.data.value.totalMoves }} moves since {{ shortDate(stats.data.value.since) }}
          </p>
          <LeagueBars :stats="stats.data.value" />
        </div>
        <dl class="grid grid-cols-2 gap-6 rounded-panel bg-surface p-6">
          <div>
            <dt class="type-body-sm text-text-muted">Free agents</dt>
            <dd class="type-data-tabular">{{ stats.data.value.freeAgents }} players</dd>
          </div>
          <div>
            <dt class="type-body-sm text-text-muted">Rumors confirmed</dt>
            <dd class="type-data-tabular">
              {{
                stats.data.value.rumorConfirmationRate === null
                  ? "None yet"
                  : `${Math.round(stats.data.value.rumorConfirmationRate * 100)}%`
              }}
            </dd>
          </div>
        </dl>
        <div v-if="stats.data.value.topTeamsByChurn.length" class="rounded-panel bg-surface p-6">
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
      </template>
    </aside>
  </div>

  <section class="mt-12" aria-labelledby="flows-heading">
    <div class="mb-4 flex flex-wrap items-center justify-between gap-4">
      <h2 id="flows-heading" class="type-heading-md">Where players went</h2>
      <div class="flex gap-2" role="group" aria-label="Flow level">
        <FilterChip :active="flowLevel === 'league'" @click="flowLevel = 'league'"
          >By league</FilterChip
        >
        <FilterChip :active="flowLevel === 'team'" @click="flowLevel = 'team'">By team</FilterChip>
      </div>
    </div>
    <QueryError v-if="flows.isError.value" what="flows" @retry="flows.refetch()" />
    <div v-else class="rounded-panel bg-surface p-6">
      <div v-if="flows.isPending.value" class="h-80" aria-busy="true" />
      <p v-else-if="!flows.data.value?.links.length" class="text-text-muted">
        No team-to-team moves this window yet.
      </p>
      <template v-else>
        <p class="mb-4 max-w-prose type-body-sm text-text-muted">
          {{ teamToTeam }} {{ teamToTeam === 1 ? "move" : "moves" }}
          <template v-if="stats.data.value">of {{ stats.data.value.totalMoves }}</template>
          went straight from one team to another. Releases, retirements and free agent signings are
          not shown.
          <template v-if="flowLevel === 'league'">
            Other covers teams outside LCK, LPL, LEC and LCS.
          </template>
        </p>
        <FlowsChart :links="flows.data.value.links" />
      </template>
    </div>
  </section>

  <section class="mt-12" aria-labelledby="rumors-heading">
    <div class="mb-4 flex flex-wrap items-baseline justify-between gap-4">
      <h2 id="rumors-heading" class="type-heading-md">Rumors to watch</h2>
      <RouterLink to="/rumors" class="type-body-sm underline hover:text-primary">
        Rumor tracker
      </RouterLink>
    </div>
    <QueryError v-if="rumors.isError.value" what="rumors" @retry="rumors.refetch()" />
    <MoveBoard
      v-else
      :items="rumors.data.value?.items"
      :pending="rumors.isPending.value"
      empty="No open rumors right now."
    />
  </section>
</template>
