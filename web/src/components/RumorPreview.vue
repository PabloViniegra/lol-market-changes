<script setup lang="ts">
import type { TeamsResponse } from "@lol/shared";
import { computed } from "vue";
import { KIND_LABELS, KINDS, playerHandle, teamCode } from "../format";
import BoardFlap from "./BoardFlap.vue";
import RoleIcon from "./RoleIcon.vue";
import StatusPill from "./StatusPill.vue";

const props = defineProps<{
  player: string;
  fromTeam: string;
  toTeam: string;
  role: string;
  kind: string;
  reliability: number;
  teams: TeamsResponse["teams"];
}>();

function code(typed: string): string | null {
  const name = typed.trim();

  if (!name) return null;

  const key = name.toLowerCase();

  const known = props.teams.find(
    (team) => team.name.toLowerCase() === key || team.short?.toLowerCase() === key,
  );

  return known ? teamCode(known.name, known.short) : teamCode(name, null);
}

const from = computed(() => code(props.fromTeam));
const to = computed(() => code(props.toTeam));
const kindLabel = computed(() => KINDS.find((item) => item === props.kind));
</script>

<template>
  <section aria-labelledby="rumor-preview-heading" class="rounded-panel bg-surface p-6">
    <h2 id="rumor-preview-heading" class="type-label-md text-text-muted">
      How it will look on the board
    </h2>
    <div class="mt-4 flex items-center gap-2">
      <BoardFlap :code="from" placeholder="FA" empty-label="free agent" :team="null" />
      <span class="text-text-muted" aria-hidden="true">▸</span>
      <span class="sr-only">to</span>
      <BoardFlap :code="to" placeholder="OUT" empty-label="left team" :team="null" />
    </div>
    <p class="mt-4 wrap-anywhere type-board-player" :class="{ 'text-text-muted': !player.trim() }">
      {{ player.trim() ? playerHandle(player.trim()) : "Player" }}
    </p>
    <p class="type-body-sm text-text-muted">
      {{ fromTeam.trim() || "Free agent" }} ▸ {{ toTeam.trim() || "No team" }}
    </p>
    <div class="mt-4 flex flex-wrap items-center gap-3 type-body-sm">
      <span v-if="role || kindLabel" class="flex items-center gap-1 text-text-muted">
        <RoleIcon :role="role || null" />
        <span v-if="role">{{ role }}</span>
        <span v-if="kindLabel">{{ role ? "· " : "" }}{{ KIND_LABELS[kindLabel] }}</span>
      </span>
      <StatusPill status="rumor" :reliability="reliability" />
    </div>
  </section>
</template>
