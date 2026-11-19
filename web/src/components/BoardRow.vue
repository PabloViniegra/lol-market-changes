<script setup lang="ts">
import type { FeedItem } from "@lol/shared";
import { computed } from "vue";
import { kindLabel, playerHandle, relativeDay, teamCode } from "../format";
import BoardFlap from "./BoardFlap.vue";
import LeagueTag from "./LeagueTag.vue";
import PlayerPortrait from "./PlayerPortrait.vue";
import RoleIcon from "./RoleIcon.vue";
import StatusPill from "./StatusPill.vue";

const props = defineProps<{ item: FeedItem; now: Date; fresh?: boolean; hidePlayer?: boolean }>();

const muted = computed(() => props.item.status === "denied" || props.item.status === "stale");

const status = computed(() =>
  props.item.status === "official" && props.item.rumorId ? "confirmed" : props.item.status,
);

const leagues = computed(() => {
  const { fromLeague, toLeague } = props.item;

  return [...new Set([fromLeague, toLeague])].filter((league) => league !== null);
});

const source = computed(() => {
  const first = props.item.sources[0];

  if (!first) return null;

  return {
    url: first.url,
    outlet: first.outlet ?? URL.parse(first.url)?.hostname.replace(/^www\./, "") ?? "source",
  };
});

const details = computed(() =>
  [props.item.role, kindLabel(props.item.kind)].filter(Boolean).join(" · "),
);
</script>

<template>
  <li
    class="board-row relative"
    :class="{
      'text-text-muted': muted,
      'row-fresh': fresh,
      'hover:bg-surface-raised': !hidePlayer,
    }"
  >
    <div class="flex shrink-0 items-center gap-2">
      <BoardFlap
        :code="item.fromTeam ? teamCode(item.fromTeam, item.fromShort) : null"
        placeholder="FA"
        empty-label="free agent"
        :team="item.fromTeam"
        :logo="item.fromLogo"
        :flip="fresh"
      />
      <span class="text-text-muted" aria-hidden="true">▸</span>
      <span class="sr-only">to</span>
      <BoardFlap
        :code="item.toTeam ? teamCode(item.toTeam, item.toShort) : null"
        placeholder="OUT"
        empty-label="left team"
        :team="item.toTeam"
        :logo="item.toLogo"
        :flip="fresh"
      />
    </div>

    <PlayerPortrait
      v-if="!hidePlayer"
      :src="item.playerImage"
      :role="item.role"
      :size="48"
      class="max-md:hidden"
      :class="{ grayscale: muted }"
    />

    <div class="min-w-0 flex-1">
      <RouterLink
        v-if="!hidePlayer"
        :to="`/players/${encodeURIComponent(item.player)}`"
        :title="item.player"
        class="row-link block wrap-anywhere type-board-player hover:text-primary"
        :class="{ 'text-text-muted': !item.toTeam }"
      >
        {{ playerHandle(item.player) }}
      </RouterLink>
      <p
        :class="hidePlayer ? 'type-body-md wrap-anywhere' : 'truncate type-body-sm text-text-muted'"
        aria-hidden="true"
      >
        {{ item.fromTeam ?? "Free agent" }} ▸ {{ item.toTeam ?? "No team" }}
      </p>
      <p v-if="item.notes" class="truncate type-body-sm text-text-muted" :title="item.notes">
        {{ item.notes }}
      </p>
    </div>

    <div
      class="flex w-full flex-wrap items-center gap-x-3 gap-y-2 type-body-sm md:w-auto md:shrink-0 md:justify-end"
    >
      <span v-if="details" class="flex items-center gap-1 text-text-muted">
        <RoleIcon :role="item.role" />
        {{ details }}
      </span>
      <span v-if="leagues.length" class="flex items-center gap-1">
        <LeagueTag v-for="league in leagues" :key="league" :league="league" />
      </span>
      <StatusPill :status="status" :reliability="item.reliability" />
      <span class="ml-auto flex max-w-full items-center gap-2 md:ml-0">
        <time :datetime="item.date" class="text-right text-text-muted tabular-nums md:w-24">
          {{ relativeDay(item.date, now) }}
        </time>
        <a
          v-if="source"
          :href="source.url"
          target="_blank"
          rel="noopener noreferrer"
          :title="`Source: ${source.outlet}`"
          class="flex min-h-11 min-w-11 items-center justify-center gap-1 rounded-control px-1 py-1 text-text-muted hover:bg-line hover:text-text md:min-h-8 md:min-w-8"
        >
          <svg
            viewBox="0 0 20 20"
            class="size-5 shrink-0"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path
              d="M11 3.5h5.5V9M16.5 3.5 9 11M14 12v3.5a1 1 0 0 1-1 1H4.5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1H8"
            />
          </svg>
          <span class="sr-only">Source: </span>
          <span
            :class="
              item.status === 'official' || item.rumorId
                ? 'sr-only'
                : 'max-w-44 wrap-anywhere underline md:max-w-24'
            "
          >
            {{ source.outlet }}
          </span>
          <span class="sr-only"> (opens in a new tab)</span>
        </a>
        <span v-else class="hidden w-8 md:block" aria-hidden="true" />
      </span>
    </div>
  </li>
</template>

<style scoped>
.row-link::before {
  content: "";
  position: absolute;
  inset: 0;
}

.row-link:focus-visible {
  outline: none;
}

li:has(.row-link:focus-visible) {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
  z-index: 1;
}

li :deep(a:not(.row-link)) {
  position: relative;
  z-index: 1;
}

@media (prefers-reduced-motion: reduce) {
  .row-fresh::before {
    content: "";
    position: absolute;
    inset-block: 0;
    left: 0;
    width: 4px;
    background-color: var(--color-primary);
    pointer-events: none;
    animation: fresh-edge 1s steps(1, end) forwards;
  }
}

@keyframes fresh-edge {
  to {
    opacity: 0;
  }
}
</style>
