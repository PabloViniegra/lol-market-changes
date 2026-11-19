<script setup lang="ts">
import { computed } from "vue";
import { HttpError } from "../api/client";
import { usePlayer } from "../api/players";
import BoardFlap from "../components/BoardFlap.vue";
import FlapSkeleton from "../components/FlapSkeleton.vue";
import MoveBoard from "../components/MoveBoard.vue";
import QueryError from "../components/QueryError.vue";
import { playerHandle, teamCode } from "../format";

const props = defineProps<{ name: string }>();

const player = usePlayer(() => props.name);

const notFound = computed(
  () => player.error.value instanceof HttpError && player.error.value.status === 404,
);

const details = computed(() => {
  const info = player.data.value?.player;

  return [info?.realName, info?.country, info?.role].filter(Boolean).join(" · ");
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
    <div class="mb-10 flex flex-wrap items-end justify-between gap-6" aria-hidden="true">
      <div>
        <span class="block h-9 w-48 max-w-full rounded-flap bg-surface-raised" />
        <span class="mt-3 block h-3 w-56 max-w-full rounded-flap bg-surface-raised" />
      </div>
      <div class="flex items-center gap-3">
        <span class="h-3 w-12 rounded-flap bg-surface-raised" />
        <FlapSkeleton />
        <span class="h-4 w-24 rounded-flap bg-surface-raised" />
      </div>
    </div>
    <span class="mb-4 block h-6 w-20 rounded-flap bg-surface-raised" aria-hidden="true" />
    <MoveBoard :items="undefined" pending :rows="4" empty="" />
  </div>
  <template v-else-if="player.data.value">
    <header class="mb-10 flex flex-wrap items-end justify-between gap-6">
      <div>
        <h1 class="type-heading-lg">{{ playerHandle(player.data.value.player.name) }}</h1>
        <p v-if="details" class="mt-1 type-body-sm text-text-muted">{{ details }}</p>
      </div>
      <div class="flex items-center gap-3">
        <span class="type-body-sm text-text-muted">
          {{ player.data.value.currentTeam ? "Now at" : "Now" }}
        </span>
        <BoardFlap
          :code="
            player.data.value.currentTeam
              ? teamCode(player.data.value.currentTeam, player.data.value.currentTeamShort)
              : null
          "
          placeholder="FA"
          empty-label="free agent"
          :team="player.data.value.currentTeam"
        />
        <span class="type-body-md">{{ player.data.value.currentTeam ?? "Free agent" }}</span>
      </div>
    </header>

    <section aria-labelledby="player-moves-heading">
      <h2 id="player-moves-heading" class="mb-4 type-heading-md">Moves</h2>
      <MoveBoard
        :key="name"
        :items="player.data.value.timeline"
        :pending="false"
        empty="No official moves in the last two seasons."
      />
    </section>

    <section class="mt-12" aria-labelledby="player-rumors-heading">
      <h2 id="player-rumors-heading" class="mb-4 type-heading-md">Rumors</h2>
      <MoveBoard
        :key="name"
        :items="player.data.value.rumors"
        :pending="false"
        empty="No rumors involve this player."
      />
    </section>
  </template>
</template>
