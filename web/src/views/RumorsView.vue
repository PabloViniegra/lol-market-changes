<script setup lang="ts">
import type { FeedStatus } from "@lol/shared";
import { computed } from "vue";
import { useFeed } from "../api/feed";
import { useStats } from "../api/stats";
import BoardLegend from "../components/BoardLegend.vue";
import MoveBoard from "../components/MoveBoard.vue";
import QueryError from "../components/QueryError.vue";
import StatusPill from "../components/StatusPill.vue";

const open = useFeed({ status: "rumor", limit: 50 });
const confirmed = useFeed({ status: "confirmed", limit: 50 });
const closed = useFeed({ status: "denied,stale", limit: 50 });
const stats = useStats("season");

const rate = computed(() => {
  const value = stats.data.value?.rumorConfirmationRate;

  return value == null
    ? null
    : `${Math.round(value * 100)}% of resolved rumors reported this window were confirmed.`;
});

const sections = [
  {
    id: "open",
    title: "Open",
    query: open,
  },
  {
    id: "confirmed",
    title: "Confirmed",
    query: confirmed,
  },
  {
    id: "closed",
    title: "Denied and faded",
    query: closed,
  },
];

const LIFECYCLE: { status: FeedStatus; text: string }[] = [
  { status: "rumor", text: "Reported, not official yet." },
  { status: "confirmed", text: "An official move matched it." },
  { status: "denied", text: "Someone involved denied it." },
  { status: "stale", text: "No confirmation after 60 days." },
];

const shown = computed(() =>
  sections.filter(({ query }) => query.isError.value || query.data.value?.items.length !== 0),
);
</script>

<template>
  <div class="mb-6 flex flex-wrap items-baseline justify-between gap-4">
    <div>
      <h1 class="type-heading-lg">Rumor tracker</h1>
      <p v-if="rate" class="mt-1 type-body-md text-text-muted">{{ rate }}</p>
    </div>
    <div class="flex flex-wrap items-center gap-4">
      <BoardLegend v-if="shown.length" />
      <RouterLink
        to="/rumors/new"
        class="inline-flex h-10 items-center rounded-control bg-surface-raised px-4 type-label-md hover:bg-line"
      >
        Send a rumor
      </RouterLink>
    </div>
  </div>

  <section
    v-for="(section, index) in shown"
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
    />
  </section>

  <section
    v-if="!shown.length"
    class="rounded-panel bg-surface p-6"
    aria-labelledby="rumors-empty-heading"
  >
    <h2 id="rumors-empty-heading" class="type-heading-md">No rumors tracked yet</h2>
    <p class="mt-2 type-body-md text-text-muted">
      Rumors come from reports, are reviewed by hand and are checked against official moves on every
      sync. Each one moves through these statuses:
    </p>
    <ul class="mt-4 flex flex-col gap-3 type-body-sm text-text-muted">
      <li v-for="step in LIFECYCLE" :key="step.status" class="flex items-center gap-3">
        <StatusPill :status="step.status" />
        <span>{{ step.text }}</span>
      </li>
    </ul>
  </section>
</template>
