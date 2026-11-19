<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";
import { useFeedPages } from "../api/feed";
import { findTeam, useTeams } from "../api/teams";
import BoardLegend from "../components/BoardLegend.vue";
import FeedFilterFields from "../components/FeedFilterFields.vue";
import MoveBoard from "../components/MoveBoard.vue";
import QueryError from "../components/QueryError.vue";
import { useMediaQuery } from "../composables/useMediaQuery";
import { KIND_LABELS, shortDate, STATUS_LABELS } from "../format";
import { DEFAULT_STATUSES, isDefaultStatuses, useFeedFilters } from "../stores/feedFilters";

const store = useFeedFilters();
const { filters, apiParams, isFiltered, hasNonTeamFilters } = storeToRefs(store);
const feed = useFeedPages(apiParams);
const items = computed(() => feed.data.value?.pages.flatMap((page) => page.items));
const teams = useTeams();
const isMobile = useMediaQuery("(max-width: 767.98px)");
const sheet = useTemplateRef<HTMLDialogElement>("sheet");

const knownTeam = computed(() =>
  filters.value.team ? findTeam(teams.data.value?.teams, filters.value.team) : undefined,
);

const teamName = computed(() => knownTeam.value?.name ?? filters.value.team);

const listFormat = new Intl.ListFormat("en", { type: "conjunction" });

const filterHint = computed(() => {
  const { league, role, kind, player, statuses, from, to } = filters.value;

  const removable = (
    [
      [league, "the league"],
      [role, "the role"],
      [kind, "the move type"],
      [player, "the player"],
      [!isDefaultStatuses(statuses), "the status filter"],
    ] as const
  ).flatMap(([active, label]) => (active ? [label] : []));

  const actions = [
    removable.length ? `removing ${listFormat.format(removable)}` : null,
    from || to ? "widening the dates" : null,
  ].filter((item) => item !== null);

  return actions.length ? ` Try ${actions.join(" or ")}.` : "";
});

const emptyMessage = computed(() => {
  if (!filters.value.team) return `No moves match these filters.${filterHint.value}`;

  if (teams.data.value && !knownTeam.value) {
    return `“${teamName.value}” isn't a tracked team. Pick one from the suggestions, by name or code, for example “Gen.G” or “FNC”.`;
  }

  if (hasNonTeamFilters.value) {
    return `No moves involve “${teamName.value}” with these filters.${filterHint.value}`;
  }

  return `No moves involve “${teamName.value}” yet.`;
});

const activeFilters = computed(() => {
  const { league, statuses, role, kind, player, from, to } = filters.value;
  const active: { key: string; label: string; clear: () => void }[] = [];

  if (isMobile.value && league) {
    active.push({ key: "league", label: league, clear: () => store.update({ league: null }) });
  }

  if (isMobile.value && !isDefaultStatuses(statuses)) {
    active.push({
      key: "status",
      label: statuses.map((status) => STATUS_LABELS[status]).join(", "),
      clear: () => store.update({ statuses: DEFAULT_STATUSES }),
    });
  }

  if (role) active.push({ key: "role", label: role, clear: () => store.update({ role: null }) });

  if (kind) {
    active.push({
      key: "kind",
      label: KIND_LABELS[kind],
      clear: () => store.update({ kind: null }),
    });
  }

  if (player) {
    active.push({
      key: "player",
      label: `Player “${player}”`,
      clear: () => store.update({ player: null }),
    });
  }

  if (teamName.value) {
    active.push({ key: "team", label: teamName.value, clear: () => store.update({ team: null }) });
  }

  if (from) {
    active.push({
      key: "from",
      label: `From ${shortDate(from, true)}`,
      clear: () => store.update({ from: null }),
    });
  }

  if (to) {
    active.push({
      key: "to",
      label: `To ${shortDate(to, true)}`,
      clear: () => store.update({ to: null }),
    });
  }

  return active;
});

function focusPlayerSearch(event: KeyboardEvent) {
  if (event.key !== "/" || !event.ctrlKey || event.altKey || event.metaKey) return;

  const target = event.target;

  if (
    target instanceof HTMLElement &&
    target.closest("input, textarea, select, [contenteditable]")
  ) {
    return;
  }

  const input = document.getElementById("player-search");

  if (!input) return;

  event.preventDefault();
  input.focus();
}

onMounted(() => window.addEventListener("keydown", focusPlayerSearch));

onUnmounted(() => window.removeEventListener("keydown", focusPlayerSearch));

const closeOnBackdrop = (event: MouseEvent) => {
  if (event.target === sheet.value) sheet.value?.close();
};
</script>

<template>
  <div class="mb-6 flex flex-wrap items-baseline justify-between gap-4">
    <h1 class="type-heading-lg">Market feed</h1>
    <p v-if="teamName" class="min-w-0 type-body-md wrap-anywhere">
      Moves involving
      <RouterLink
        v-if="knownTeam"
        :to="`/teams/${encodeURIComponent(knownTeam.name)}`"
        class="underline hover:text-primary"
      >
        {{ knownTeam.name }}
      </RouterLink>
      <template v-else>“{{ teamName }}”</template>
    </p>
  </div>

  <dialog
    v-if="isMobile"
    ref="sheet"
    aria-labelledby="filters-heading"
    class="filter-sheet mx-0 mt-auto mb-0 max-h-5/6 w-full max-w-full overscroll-contain scroll-pt-18 rounded-t-panel bg-surface-raised text-text backdrop:bg-background/60"
    @click="closeOnBackdrop"
  >
    <form @submit.prevent>
      <div
        class="sticky top-0 z-10 flex items-center justify-between gap-4 border-b border-line bg-surface-raised px-4 py-3"
      >
        <h2 id="filters-heading" class="type-heading-md">Filters</h2>
        <button
          type="button"
          class="h-11 rounded-control bg-primary px-4 type-label-md text-on-primary hover:bg-primary-hover"
          @click="sheet?.close()"
        >
          Show moves
        </button>
      </div>
      <div class="p-4">
        <FeedFilterFields />
      </div>
    </form>
  </dialog>
  <form v-else class="mb-4 rounded-panel bg-surface p-4" aria-label="Filters" @submit.prevent>
    <FeedFilterFields collapsible />
  </form>

  <div
    v-if="isMobile || activeFilters.length || isFiltered"
    class="mb-4 flex flex-wrap items-center gap-2 md:pl-28"
  >
    <button
      v-if="isMobile"
      type="button"
      aria-haspopup="dialog"
      class="flex h-11 items-center gap-2 rounded-control bg-surface-raised px-4 type-label-md hover:bg-line"
      @click="sheet?.showModal()"
    >
      <svg
        viewBox="0 0 20 20"
        class="size-5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        aria-hidden="true"
      >
        <path d="M3.5 6h13M6 10h8M8.5 14h3" />
      </svg>
      Filters
    </button>
    <button
      v-for="filter in activeFilters"
      :key="filter.key"
      type="button"
      :aria-label="`Remove filter: ${filter.label}`"
      class="flex min-h-11 max-w-full items-center gap-1 rounded-control bg-text pr-2 pl-3 type-label-md wrap-anywhere text-background hover:bg-text-muted md:min-h-8"
      @click="filter.clear()"
    >
      {{ filter.label }}
      <svg
        viewBox="0 0 20 20"
        class="size-5 shrink-0"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        aria-hidden="true"
      >
        <path d="m6 6 8 8m0-8-8 8" />
      </svg>
    </button>
    <button
      v-if="isFiltered"
      type="button"
      class="h-11 rounded-control px-3 type-label-md text-text-muted underline hover:text-text md:h-8"
      @click="store.clear()"
    >
      Clear filters
    </button>
  </div>

  <div class="mb-2 flex flex-wrap items-baseline justify-between gap-x-4">
    <p class="type-body-sm text-text-muted" aria-live="polite" aria-atomic="true">
      <template v-if="!feed.isError.value && !feed.isPending.value">
        <template v-if="items?.length">
          Showing {{ items.length }} {{ items.length === 1 ? "move" : "moves" }}, newest first
        </template>
        <span v-else class="sr-only">{{ emptyMessage }}</span>
      </template>
    </p>
    <BoardLegend v-if="!feed.isError.value" />
  </div>
  <QueryError v-if="feed.isError.value" what="moves" @retry="feed.refetch()" />
  <template v-else>
    <div v-if="feed.hasNextPage.value" class="relative">
      <a
        href="#load-more"
        class="sr-only focus:not-sr-only focus:absolute focus:z-10 focus:rounded-control focus:bg-surface-raised focus:p-3"
        >Skip to load more moves</a
      >
    </div>
    <MoveBoard
      :key="JSON.stringify(apiParams)"
      :items="items"
      :pending="feed.isPending.value"
      :empty="emptyMessage"
    />
    <button
      v-if="feed.hasNextPage.value"
      id="load-more"
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
@supports (transition-behavior: allow-discrete) {
  .filter-sheet {
    opacity: 0;
    transition:
      opacity var(--duration-sheet) var(--ease-drawer),
      transform var(--duration-sheet) var(--ease-drawer),
      display var(--duration-sheet) allow-discrete,
      overlay var(--duration-sheet) allow-discrete;
  }

  .filter-sheet::backdrop {
    opacity: 0;
    transition:
      opacity var(--duration-sheet) var(--ease-drawer),
      display var(--duration-sheet) allow-discrete,
      overlay var(--duration-sheet) allow-discrete;
  }

  .filter-sheet[open],
  .filter-sheet[open]::backdrop {
    opacity: 1;
  }

  .filter-sheet:not([open]),
  .filter-sheet:not([open])::backdrop {
    pointer-events: none;
  }

  @starting-style {
    .filter-sheet[open],
    .filter-sheet[open]::backdrop {
      opacity: 0;
    }
  }

  @media (prefers-reduced-motion: no-preference) {
    .filter-sheet {
      transform: translateY(100%);
    }

    .filter-sheet[open] {
      transform: translateY(0);
    }

    @starting-style {
      .filter-sheet[open] {
        transform: translateY(100%);
      }
    }
  }
}
</style>
