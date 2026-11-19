<script setup lang="ts">
import type { League, StatsResponse } from "@lol/shared";
import { computed } from "vue";
import { LEAGUES } from "../format";

const props = defineProps<{ stats: StatsResponse }>();

const BAR_CLASS: Record<League, string> = {
  LCK: "bg-league-lck",
  LPL: "bg-league-lpl",
  LEC: "bg-league-lec",
  LCS: "bg-league-lcs",
};

const rows = computed(() => {
  const totals = LEAGUES.map((league) => ({
    league,
    count: props.stats.byLeagueAndKind
      .filter((row) => row.league === league)
      .reduce((sum, row) => sum + row.count, 0),
  }));

  const max = Math.max(1, ...totals.map((row) => row.count));

  return totals.map((row) => ({ ...row, width: `${(row.count / max) * 100}%` }));
});
</script>

<template>
  <ul class="flex flex-col">
    <li v-for="row in rows" :key="row.league">
      <RouterLink
        :to="{ path: '/feed', query: { league: row.league } }"
        class="flex min-h-11 items-center gap-3 rounded-control hover:text-primary md:min-h-7"
      >
        <span class="w-9 type-label-md underline">{{ row.league }}</span>
        <span class="sr-only">moves, {{ row.count }} this window</span>
        <span class="h-3 flex-1" aria-hidden="true">
          <span
            class="block h-full rounded-flap"
            :class="BAR_CLASS[row.league]"
            :style="{ width: row.width }"
          />
        </span>
        <span class="w-8 text-right type-data-tabular" aria-hidden="true">{{ row.count }}</span>
      </RouterLink>
    </li>
  </ul>
</template>
