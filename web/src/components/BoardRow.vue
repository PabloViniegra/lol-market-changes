<script setup lang="ts">
import type { FeedItem } from "@lol/shared";
import { computed } from "vue";
import { kindLabel, playerHandle, relativeDay, teamCode } from "../format";
import BoardFlap from "./BoardFlap.vue";
import LeagueTag from "./LeagueTag.vue";
import StatusPill from "./StatusPill.vue";

const props = defineProps<{ item: FeedItem; now: Date; fresh?: boolean }>();

const muted = computed(() => props.item.status === "denied" || props.item.status === "stale");

const leagues = computed(() => {
  const { fromLeague, toLeague } = props.item;

  return [...new Set([fromLeague, toLeague])].filter((league) => league !== null);
});

const details = computed(() =>
  [props.item.role, kindLabel(props.item.kind)].filter(Boolean).join(" · "),
);
</script>

<template>
  <li
    class="relative flex flex-wrap items-center gap-x-4 gap-y-2 bg-surface p-3 hover:bg-surface-raised md:min-h-16 md:flex-nowrap"
    :class="{ 'text-text-muted': muted, 'row-fresh': fresh }"
  >
    <div class="flex shrink-0 items-center gap-2">
      <BoardFlap
        :code="item.fromTeam ? teamCode(item.fromTeam, item.fromShort) : null"
        placeholder="FA"
        empty-label="free agent"
        :team="item.fromTeam"
        :flip="fresh"
      />
      <span class="text-text-muted" aria-hidden="true">▸</span>
      <span class="sr-only">to</span>
      <BoardFlap
        :code="item.toTeam ? teamCode(item.toTeam, item.toShort) : null"
        placeholder="OUT"
        empty-label="left team"
        :team="item.toTeam"
        :flip="fresh"
      />
    </div>

    <div class="min-w-0 flex-1">
      <p class="truncate type-board-player" :title="item.player">{{ playerHandle(item.player) }}</p>
      <p v-if="item.notes" class="truncate type-body-sm text-text-muted" :title="item.notes">
        {{ item.notes }}
      </p>
    </div>

    <div
      class="flex w-full flex-wrap items-center gap-x-3 gap-y-2 type-body-sm md:w-auto md:shrink-0 md:justify-end"
    >
      <span v-if="details" class="text-text-muted">{{ details }}</span>
      <span v-if="leagues.length" class="flex items-center gap-1">
        <LeagueTag v-for="league in leagues" :key="league" :league="league" />
      </span>
      <StatusPill :status="item.status" :reliability="item.reliability" />
      <time :datetime="item.date" class="text-text-muted md:w-16 md:text-right">
        {{ relativeDay(item.date, now) }}
      </time>
      <a
        v-if="item.sources[0]"
        :href="item.sources[0].url"
        target="_blank"
        rel="noopener noreferrer"
        class="text-text-muted underline hover:text-text"
        >{{ item.sources[0].outlet ?? "Source" }}</a
      >
    </div>
  </li>
</template>

<style scoped>
@media (prefers-reduced-motion: reduce) {
  .row-fresh::before {
    content: "";
    position: absolute;
    inset-block: 0;
    left: 0;
    width: 4px;
    background-color: var(--color-primary);
    animation: fresh-edge 1s steps(1, end) forwards;
  }
}

@keyframes fresh-edge {
  to {
    opacity: 0;
  }
}
</style>
