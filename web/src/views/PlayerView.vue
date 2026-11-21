<script setup lang="ts">
import type { FeedItem } from "@lol/shared";
import { computed } from "vue";
import { HttpError } from "../api/client";
import { usePlayer } from "../api/players";
import BoardFlap from "../components/BoardFlap.vue";
import FlapSkeleton from "../components/FlapSkeleton.vue";
import MoveBoard from "../components/MoveBoard.vue";
import PlayerPortrait from "../components/PlayerPortrait.vue";
import QueryError from "../components/QueryError.vue";
import RoleIcon from "../components/RoleIcon.vue";
import StatusPill from "../components/StatusPill.vue";
import { useNow } from "../composables/useNow";
import { playerHandle, relativeDay, shortDate, teamCode } from "../format";

const props = defineProps<{ name: string }>();

const player = usePlayer(() => props.name);
const now = useNow();

const notFound = computed(
  () => player.error.value instanceof HttpError && player.error.value.status === 404,
);

const details = computed(() => {
  const info = player.data.value?.player;

  return [info?.realName, info?.country, info?.role].filter(Boolean).join(" · ");
});

const latestMove = computed(() => player.data.value?.timeline[0]);

const retired = computed(
  () => !player.data.value?.currentTeam && latestMove.value?.kind === "retirement",
);

const standing = computed(() => {
  const data = player.data.value;
  const move = latestMove.value;

  if (!data || !move || move.toTeam !== data.currentTeam) return null;

  const since = `Since ${shortDate(move.date, move.date.slice(0, 4) !== String(now.value.getUTCFullYear()))}`;

  return !data.currentTeam && move.fromTeam ? `${since} · left ${move.fromTeam}` : since;
});

const topRumor = computed(
  () =>
    player.data.value?.rumors
      .filter((rumor) => rumor.status === "rumor")
      .toSorted(
        (a, b) => (b.reliability ?? 0) - (a.reliability ?? 0) || b.date.localeCompare(a.date),
      )[0],
);

function rumorTarget(rumor: FeedItem): string {
  if (rumor.toTeam) return `to ${rumor.toTeam}`;

  return rumor.fromTeam ? `to leave ${rumor.fromTeam}` : "to leave their team";
}

// Same rule as the feed: a confirmed rumor is shown through its official move when both exist.
const history = computed(() => {
  const data = player.data.value;

  if (!data) return [];

  const official = new Set(data.timeline.map((move) => move.id));

  const rumors = data.rumors.filter(
    (rumor) =>
      !(rumor.status === "confirmed" && rumor.transferId && official.has(rumor.transferId)),
  );

  return [...data.timeline, ...rumors].toSorted(
    (a, b) => b.date.localeCompare(a.date) || b.id.localeCompare(a.id),
  );
});
</script>

<template>
  <div v-if="notFound" class="max-w-prose">
    <h1 class="mb-2 type-heading-lg">Player not found</h1>
    <p class="text-text-muted">
      No tracked player is called “{{ name }}”. Player names follow Leaguepedia, for example
      “Faker”.
    </p>
  </div>
  <QueryError v-else-if="player.isError.value" what="this player" @retry="player.refetch()" />
  <div v-else-if="player.isPending.value" aria-busy="true">
    <span class="sr-only">Loading player</span>
    <div class="mb-10 flex flex-wrap items-end justify-between gap-x-6 gap-y-5" aria-hidden="true">
      <div class="flex items-end gap-4 md:gap-6">
        <span class="size-28 shrink-0 rounded-flap bg-surface-raised" />
        <div>
          <span class="block h-9 w-48 max-w-full rounded-flap bg-surface-raised" />
          <span class="mt-3 block h-3 w-56 max-w-full rounded-flap bg-surface-raised" />
        </div>
      </div>
      <div class="flex items-center gap-3">
        <FlapSkeleton />
        <div>
          <span class="block h-4 w-32 rounded-flap bg-surface-raised" />
          <span class="mt-2 block h-3 w-24 rounded-flap bg-surface-raised" />
        </div>
      </div>
    </div>
    <span class="mb-4 block h-6 w-44 rounded-flap bg-surface-raised" aria-hidden="true" />
    <MoveBoard :items="undefined" pending :rows="4" hide-player />
  </div>
  <template v-else-if="player.data.value">
    <header class="mb-10 flex flex-wrap items-end justify-between gap-x-6 gap-y-5">
      <div class="flex min-w-0 items-end gap-4 md:gap-6">
        <PlayerPortrait
          :src="player.data.value.player.imageUrl"
          :role="player.data.value.player.role"
          :size="112"
        />
        <div class="min-w-0">
          <h1 class="type-heading-lg wrap-anywhere">
            {{ playerHandle(player.data.value.player.name) }}
          </h1>
          <p v-if="details" class="mt-1 flex items-center gap-1 type-body-sm text-text-muted">
            <RoleIcon :role="player.data.value.player.role" />
            {{ details }}
          </p>
        </div>
      </div>

      <div class="flex min-w-0 flex-col gap-3 md:items-end">
        <div class="flex min-w-0 items-center gap-3">
          <BoardFlap
            :code="
              player.data.value.currentTeam
                ? teamCode(player.data.value.currentTeam, player.data.value.currentTeamShort)
                : null
            "
            :placeholder="retired ? 'RET' : 'FA'"
            :empty-label="retired ? 'retired' : 'free agent'"
            :team="player.data.value.currentTeam"
            :logo="player.data.value.currentTeamLogo"
          />
          <div class="min-w-0">
            <p class="type-body-md wrap-anywhere">
              <span class="sr-only">Now: </span>
              <RouterLink
                v-if="player.data.value.currentTeam"
                :to="`/teams/${encodeURIComponent(player.data.value.currentTeam)}`"
                class="underline hover:text-primary"
              >
                {{ player.data.value.currentTeam }}
              </RouterLink>
              <template v-else>{{ retired ? "Retired" : "Free agent" }}</template>
            </p>
            <p v-if="standing" class="type-body-sm text-text-muted wrap-anywhere">
              {{ standing }}
            </p>
          </div>
        </div>
        <p v-if="topRumor" class="flex flex-wrap items-center gap-x-2 gap-y-1 type-body-sm">
          <StatusPill status="rumor" :reliability="topRumor.reliability" />
          <span class="wrap-anywhere">{{ rumorTarget(topRumor) }}</span>
          <time :datetime="topRumor.date" class="text-text-muted">
            {{ relativeDay(topRumor.date, now) }}
          </time>
        </p>
      </div>
    </header>

    <section aria-labelledby="player-history-heading">
      <h2 id="player-history-heading" class="mb-4 type-heading-md">Moves and rumors</h2>
      <MoveBoard
        :key="name"
        :items="history"
        :pending="false"
        hide-player
        empty="No moves or rumors involve this player in the last two seasons."
      />
      <p v-if="history.length" class="mt-3 type-body-sm text-text-muted">
        Official moves go back two seasons.
      </p>
    </section>
  </template>
</template>
