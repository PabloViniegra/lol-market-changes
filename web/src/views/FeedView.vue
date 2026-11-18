<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useFeedPages } from "../api/feed";
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
    <fieldset class="flex flex-wrap items-center gap-2">
      <legend class="sr-only">League</legend>
      <FilterChip
        v-for="league in LEAGUES"
        :key="league"
        :active="filters.league === league"
        @click="toggle('league', league)"
      >
        {{ league }}
      </FilterChip>
      <span class="mx-2 h-6 w-px bg-line" aria-hidden="true" />
      <FilterChip
        v-for="role in ROLES"
        :key="role"
        :active="filters.role === role"
        @click="toggle('role', role)"
      >
        {{ role }}
      </FilterChip>
    </fieldset>

    <fieldset class="flex flex-wrap items-center gap-2">
      <legend class="sr-only">Move type</legend>
      <FilterChip
        v-for="kind in KINDS"
        :key="kind"
        :active="filters.kind === kind"
        @click="toggle('kind', kind)"
      >
        {{ KIND_LABELS[kind] }}
      </FilterChip>
    </fieldset>

    <div class="flex flex-wrap items-end gap-x-6 gap-y-4">
      <fieldset class="flex flex-wrap items-center gap-2">
        <legend class="sr-only">Status</legend>
        <FilterChip
          v-for="status in STATUSES"
          :key="status"
          :active="filters.statuses.includes(status)"
          @click="store.toggleStatus(status)"
        >
          {{ STATUS_LABELS[status] }}
        </FilterChip>
      </fieldset>

      <label class="flex flex-col gap-1 type-body-sm text-text-muted">
        From
        <input
          type="date"
          :value="filters.from ?? ''"
          class="h-10 rounded-control bg-surface-raised px-3 type-body-md text-text"
          @change="setDate('from', $event)"
        />
      </label>
      <label class="flex flex-col gap-1 type-body-sm text-text-muted">
        To
        <input
          type="date"
          :value="filters.to ?? ''"
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
