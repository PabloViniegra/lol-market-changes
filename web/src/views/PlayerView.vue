<script setup lang="ts">
import type { FeedItem, Move, Rumor } from "@lol/shared";
import { computed } from "vue";
import { HttpError } from "../api/client";
import { usePlayer } from "../api/players";
import BoardFlap from "../components/BoardFlap.vue";
import MoveBoard from "../components/MoveBoard.vue";
import QueryError from "../components/QueryError.vue";
import { playerHandle, teamCode } from "../format";

const props = defineProps<{ name: string }>();

const player = usePlayer(() => props.name);

const notFound = computed(
  () => player.error.value instanceof HttpError && player.error.value.status === 404,
);

const base = (name: string) => ({
  player: name,
  fromShort: null,
  toShort: null,
  fromLeague: null,
  toLeague: null,
  notes: null,
  rumorId: null,
  transferId: null,
});

const moves = computed<FeedItem[] | undefined>(() =>
  player.data.value?.timeline.map((move: Move) => ({
    ...base(props.name),
    id: move.id,
    status: "official",
    fromTeam: move.fromTeam,
    toTeam: move.toTeam,
    role: move.role,
    kind: move.kind,
    date: move.date,
    reliability: null,
    sources: [],
  })),
);

const rumors = computed<FeedItem[] | undefined>(() =>
  player.data.value?.rumors.map((rumor: Rumor) => ({
    ...base(props.name),
    id: rumor.id,
    status: rumor.status,
    fromTeam: rumor.fromTeam,
    toTeam: rumor.toTeam,
    role: rumor.role,
    kind: rumor.kind,
    date: rumor.reportedAt,
    reliability: rumor.reliability,
    sources: rumor.sources,
    notes: rumor.notes,
    transferId: rumor.transferId,
  })),
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
  <div v-else-if="player.isPending.value" class="h-40" aria-busy="true" />
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
            player.data.value.currentTeam ? teamCode(player.data.value.currentTeam, null) : null
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
        :items="moves"
        :pending="false"
        empty="No official moves in the last two seasons."
      />
    </section>

    <section class="mt-12" aria-labelledby="player-rumors-heading">
      <h2 id="player-rumors-heading" class="mb-4 type-heading-md">Rumors</h2>
      <MoveBoard
        :key="name"
        :items="rumors"
        :pending="false"
        empty="No rumors involve this player."
      />
    </section>
  </template>
</template>
