<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import { findTeam, useTeams } from "../api/teams";
import { KIND_LABELS, KINDS, LEAGUES, ROLES, STATUS_LABELS, STATUSES } from "../format";
import { useFeedFilters } from "../stores/feedFilters";
import FilterChip from "./FilterChip.vue";

const props = defineProps<{ collapsible?: boolean }>();

const store = useFeedFilters();
const { filters, secondaryFilterCount } = storeToRefs(store);
const teams = useTeams();
const moreOpen = ref(false);
const showMore = computed(() => !props.collapsible || moreOpen.value);
const fieldBg = computed(() => (props.collapsible ? "bg-surface-raised" : "bg-surface"));

const teamValue = computed(() =>
  filters.value.team
    ? (findTeam(teams.data.value?.teams, filters.value.team)?.name ?? filters.value.team)
    : "",
);

const toggle = <K extends "league" | "role" | "kind">(
  key: K,
  value: NonNullable<(typeof filters.value)[K]>,
) => store.update({ [key]: filters.value[key] === value ? null : value });

const setTeam = (event: Event) => {
  const input = event.target;

  if (!(input instanceof HTMLInputElement)) return;

  const value = input.value.trim();

  store.update({ team: value ? (findTeam(teams.data.value?.teams, value)?.name ?? value) : null });
};

const setDate = (key: "from" | "to", event: Event) => {
  const input = event.target;

  if (input instanceof HTMLInputElement) store.update({ [key]: input.value || null });
};
</script>

<template>
  <div class="flex flex-col gap-4">
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

    <div v-if="collapsible" class="md:pl-24">
      <button
        type="button"
        :aria-expanded="moreOpen"
        aria-controls="more-filters"
        class="-ml-2 flex h-8 items-center gap-2 rounded-control px-2 type-label-md text-text-muted hover:text-text"
        @click="moreOpen = !moreOpen"
      >
        <svg
          viewBox="0 0 20 20"
          class="size-5"
          :class="{ 'rotate-90': moreOpen }"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="m8 5 5 5-5 5" />
        </svg>
        Role, move, team and dates
        <span v-if="secondaryFilterCount" class="text-text">
          ({{ secondaryFilterCount }} active)
        </span>
      </button>
    </div>

    <div v-show="showMore" id="more-filters" class="flex flex-col gap-4">
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

      <div class="flex flex-wrap items-end gap-x-6 gap-y-4 md:pl-24">
        <label class="flex w-full flex-col gap-1 type-body-sm text-text-muted sm:w-64">
          Team
          <input
            type="search"
            :value="teamValue"
            list="tracked-teams"
            autocomplete="off"
            placeholder="Name or code, e.g. FNC"
            :class="fieldBg"
            class="h-11 rounded-control px-3 type-body-md text-text placeholder:text-text-muted md:h-10"
            @change="setTeam"
          />
        </label>
        <datalist id="tracked-teams">
          <option v-for="team in teams.data.value?.teams" :key="team.name" :value="team.name">
            {{ [team.short, team.league].filter(Boolean).join(" · ") }}
          </option>
        </datalist>
        <label class="flex flex-col gap-1 type-body-sm text-text-muted">
          From
          <input
            type="date"
            :value="filters.from ?? ''"
            :max="filters.to ?? undefined"
            :class="fieldBg"
            class="h-11 rounded-control px-3 type-body-md text-text md:h-10"
            @change="setDate('from', $event)"
          />
        </label>
        <label class="flex flex-col gap-1 type-body-sm text-text-muted">
          To
          <input
            type="date"
            :value="filters.to ?? ''"
            :min="filters.from ?? undefined"
            :class="fieldBg"
            class="h-11 rounded-control px-3 type-body-md text-text md:h-10"
            @change="setDate('to', $event)"
          />
        </label>
      </div>
    </div>
  </div>
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
