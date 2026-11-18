<script setup lang="ts">
import type { League } from "@lol/shared";

defineProps<{ league: League }>();
</script>

<template>
  <span
    class="rounded-flap px-1.5 py-0.5 type-label-md text-on-primary"
    :class="{
      'bg-league-lck': league === 'LCK',
      'bg-league-lpl': league === 'LPL',
      'bg-league-lec': league === 'LEC',
      'bg-league-lcs': league === 'LCS',
    }"
    >{{ league }}</span
  >
</template>
