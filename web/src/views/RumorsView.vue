<script setup lang="ts">
import { computed } from "vue";
import { useFeed } from "../api/feed";
import { useStats } from "../api/stats";
import BoardLegend from "../components/BoardLegend.vue";
import MoveBoard from "../components/MoveBoard.vue";
import QueryError from "../components/QueryError.vue";

const open = useFeed({ status: "rumor", limit: 50 });
const confirmed = useFeed({ status: "confirmed", limit: 50 });
const closed = useFeed({ status: "denied,stale", limit: 50 });
const stats = useStats("season");

const rate = computed(() => {
  const value = stats.data.value?.rumorConfirmationRate;

  if (value === undefined) return null;

  return value === null
    ? "No rumor has been confirmed this window yet."
    : `${Math.round(value * 100)}% of rumors this window came true.`;
});

const sections = [
  {
    id: "open",
    title: "Open",
    query: open,
    empty: "No open rumors right now.",
  },
  {
    id: "confirmed",
    title: "Confirmed",
    query: confirmed,
    empty: "No rumor has become an official move yet.",
  },
  {
    id: "closed",
    title: "Denied and faded",
    query: closed,
    empty: "No rumors have been denied or gone 60 days without confirmation.",
  },
];
</script>

<template>
  <div class="mb-6 flex flex-wrap items-baseline justify-between gap-4">
    <div>
      <h1 class="type-heading-lg">Rumor tracker</h1>
      <p v-if="rate" class="mt-1 type-body-md text-text-muted">{{ rate }}</p>
    </div>
    <BoardLegend />
  </div>

  <section
    v-for="(section, index) in sections"
    :key="section.id"
    :class="{ 'mt-12': index > 0 }"
    :aria-labelledby="`rumors-${section.id}-heading`"
  >
    <h2 :id="`rumors-${section.id}-heading`" class="mb-4 type-heading-md">{{ section.title }}</h2>
    <QueryError v-if="section.query.isError.value" what="rumors" @retry="section.query.refetch()" />
    <MoveBoard
      v-else
      :items="section.query.data.value?.items"
      :pending="section.query.isPending.value"
      :empty="section.empty"
    />
  </section>
</template>
