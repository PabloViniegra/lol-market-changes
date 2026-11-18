<script setup lang="ts">
import type { FlowsResponse } from "@lol/shared";
import { defineAsyncComponent, ref } from "vue";
import { useFeed } from "../api/feed";
import { useFlows, useStats } from "../api/stats";
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
</script>

<template>
  <h1 class="sr-only">Dashboard</h1>

  <div class="flex flex-col gap-12 lg:flex-row lg:gap-6">
    <section class="min-w-0 flex-1" aria-labelledby="latest-heading">
      <h2 id="latest-heading" class="mb-4 type-heading-md">Latest moves</h2>
      <QueryError v-if="latest.isError.value" what="moves" @retry="latest.refetch()" />
      <MoveBoard
        v-else
        :items="latest.data.value?.items"
        :pending="latest.isPending.value"
        empty="No moves yet this season. New moves show up here within 30 minutes of Leaguepedia."
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
          <h3 class="mb-3 type-label-md">Busiest teams</h3>
          <ul class="flex flex-col gap-2 type-body-sm">
            <li
              v-for="team in stats.data.value.topTeamsByChurn.slice(0, 5)"
              :key="team.team"
              class="flex justify-between gap-4"
            >
              <RouterLink
                :to="`/teams/${encodeURIComponent(team.team)}`"
                class="truncate underline hover:text-primary"
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
        No team-to-team moves this season yet.
      </p>
      <FlowsChart v-else :links="flows.data.value.links" />
    </div>
  </section>

  <section class="mt-12" aria-labelledby="rumors-heading">
    <h2 id="rumors-heading" class="mb-4 type-heading-md">Rumors to watch</h2>
    <QueryError v-if="rumors.isError.value" what="rumors" @retry="rumors.refetch()" />
    <MoveBoard
      v-else
      :items="rumors.data.value?.items"
      :pending="rumors.isPending.value"
      empty="No open rumors right now."
    />
  </section>
</template>
