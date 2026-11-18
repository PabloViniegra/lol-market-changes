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
  <ul class="flex flex-col gap-2">
    <li v-for="row in rows" :key="row.league" class="flex items-center gap-3">
      <span class="w-9 type-label-md">{{ row.league }}</span>
      <span class="h-3 flex-1">
        <span
          class="block h-full rounded-flap"
          :class="BAR_CLASS[row.league]"
          :style="{ width: row.width }"
        />
      </span>
      <span class="w-8 text-right type-data-tabular">{{ row.count }}</span>
    </li>
  </ul>
</template>
