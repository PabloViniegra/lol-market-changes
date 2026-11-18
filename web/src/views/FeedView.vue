<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useFeedPages } from "../api/feed";
import BoardLegend from "../components/BoardLegend.vue";
import FilterChip from "../components/FilterChip.vue";
import MoveBoard from "../components/MoveBoard.vue";
import QueryError from "../components/QueryError.vue";
import { KIND_LABELS, KINDS, LEAGUES, ROLES, STATUS_LABELS, STATUSES } from "../format";
import { useFeedFilters } from "../stores/feedFilters";

const store = useFeedFilters();
const { filters, apiParams, isFiltered } = storeToRefs(store);
const feed = useFeedPages(apiParams);
const items = computed(() => feed.data.value?.pages.flatMap((page) => page.items));

const toggle = <K extends "league" | "role" | "kind">(
  key: K,
  value: NonNullable<(typeof filters.value)[K]>,
) => store.update({ [key]: filters.value[key] === value ? null : value });

const setTeam = (event: Event) => {
  const input = event.target;

  if (input instanceof HTMLInputElement) store.update({ team: input.value.trim() || null });
};

const setDate = (key: "from" | "to", event: Event) => {
  const input = event.target;

  if (input instanceof HTMLInputElement) store.update({ [key]: input.value || null });
};
</script>

<template>
  <div class="mb-6 flex flex-wrap items-baseline justify-between gap-4">
    <h1 class="type-heading-lg">Market feed</h1>
    <p v-if="filters.team" class="flex items-center gap-3 type-body-md">
      Moves involving
      <RouterLink
        :to="`/teams/${encodeURIComponent(filters.team)}`"
        class="underline hover:text-primary"
      >
        {{ filters.team }}
      </RouterLink>
    </p>
  </div>

  <form class="mb-8 flex flex-col gap-4" aria-label="Filters" @submit.prevent>
    <fieldset class="filter-group">
      <legend class="filter-legend">League</legend>
      <div class="flex flex-wrap gap-2">
        <FilterChip
          v-for="league in LEAGUES"
          :key="league"
          :active="filters.league === league"
          @click="toggle('league', league)"
        >
          {{ league }}
        </FilterChip>
      </div>
    </fieldset>

    <fieldset class="filter-group">
      <legend class="filter-legend">Role</legend>
      <div class="flex flex-wrap gap-2">
        <FilterChip
          v-for="role in ROLES"
          :key="role"
          :active="filters.role === role"
          @click="toggle('role', role)"
        >
          {{ role }}
        </FilterChip>
      </div>
    </fieldset>

    <fieldset class="filter-group">
      <legend class="filter-legend">Move</legend>
      <div class="flex flex-wrap gap-2">
        <FilterChip
          v-for="kind in KINDS"
          :key="kind"
          :active="filters.kind === kind"
          @click="toggle('kind', kind)"
        >
          {{ KIND_LABELS[kind] }}
        </FilterChip>
      </div>
    </fieldset>

    <fieldset class="filter-group">
      <legend class="filter-legend">Status</legend>
      <div class="flex flex-wrap gap-2">
        <FilterChip
          v-for="status in STATUSES"
          :key="status"
          :active="filters.statuses.includes(status)"
          @click="store.toggleStatus(status)"
        >
          {{ STATUS_LABELS[status] }}
        </FilterChip>
      </div>
    </fieldset>

    <div class="flex flex-wrap items-end gap-x-6 gap-y-4 md:pl-24">
      <label class="flex flex-col gap-1 type-body-sm text-text-muted">
        Team
        <input
          type="search"
          :value="filters.team ?? ''"
          placeholder="Leaguepedia name, e.g. Gen.G"
          class="h-10 w-64 rounded-control bg-surface-raised px-3 type-body-md text-text placeholder:text-text-muted"
          @change="setTeam"
        />
      </label>
      <label class="flex flex-col gap-1 type-body-sm text-text-muted">
        From
        <input
          type="date"
          :value="filters.from ?? ''"
          :max="filters.to ?? undefined"
          class="h-10 rounded-control bg-surface-raised px-3 type-body-md text-text"
          @change="setDate('from', $event)"
        />
      </label>
      <label class="flex flex-col gap-1 type-body-sm text-text-muted">
        To
        <input
          type="date"
          :value="filters.to ?? ''"
          :min="filters.from ?? undefined"
          class="h-10 rounded-control bg-surface-raised px-3 type-body-md text-text"
          @change="setDate('to', $event)"
        />
      </label>

      <button
        v-if="isFiltered"
        type="button"
        class="h-10 rounded-control bg-surface-raised px-4 type-label-md hover:bg-line"
        @click="store.clear()"
      >
        Clear filters
      </button>
    </div>
  </form>

  <QueryError v-if="feed.isError.value" what="moves" @retry="feed.refetch()" />
  <template v-else>
    <div class="mb-2 flex flex-wrap items-baseline justify-between gap-x-4">
      <p class="type-body-sm text-text-muted" aria-live="polite">
        <template v-if="items?.length">
          Showing {{ items.length }} {{ items.length === 1 ? "move" : "moves" }}, newest first
        </template>
      </p>
      <BoardLegend />
    </div>
    <MoveBoard
      :key="JSON.stringify(apiParams)"
      :items="items"
      :pending="feed.isPending.value"
      empty="No moves match these filters. Try removing a role or widening the dates."
    />
    <button
      v-if="feed.hasNextPage.value"
      type="button"
      class="mt-4 h-10 rounded-control bg-surface-raised px-4 type-label-md hover:bg-line disabled:text-text-muted"
      :disabled="feed.isFetchingNextPage.value"
      @click="feed.fetchNextPage()"
    >
      {{ feed.isFetchingNextPage.value ? "Loading…" : "Load more moves" }}
    </button>
  </template>
</template>

<style scoped>
@reference "../style.css";

.filter-group {
  @apply relative md:pl-24;
}

.filter-legend {
  @apply mb-2 type-label-md text-text-muted md:absolute md:top-0 md:left-0 md:mb-0 md:flex md:h-8 md:items-center;
}
</style>
