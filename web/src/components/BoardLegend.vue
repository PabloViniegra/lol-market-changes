<script setup lang="ts">
import type { FeedStatus } from "@lol/shared";
import StatusPill from "./StatusPill.vue";

const STATUS_HELP: { status: FeedStatus; reliability: number | null; text: string }[] = [
  { status: "official", reliability: null, text: "Recorded on Leaguepedia." },
  { status: "confirmed", reliability: null, text: "A rumor that became an official move." },
  {
    status: "rumor",
    reliability: 3,
    text: "Reported, not official. Dots rate the source, 1 to 5.",
  },
  { status: "denied", reliability: null, text: "A rumor denied by someone involved." },
  { status: "stale", reliability: null, text: "A rumor with no confirmation after 60 days." },
];
</script>

<template>
  <details class="group type-body-sm text-text-muted open:basis-full">
    <summary
      class="inline-flex h-11 cursor-pointer list-none items-center gap-1 rounded-control hover:text-text md:h-8"
    >
      How to read the board
      <span class="group-open:rotate-90" aria-hidden="true">▸</span>
    </summary>
    <div class="mt-2 grid gap-x-8 gap-y-3 rounded-panel bg-surface p-4 md:grid-cols-2 md:p-6">
      <p class="md:col-span-2">
        <span class="type-label-md text-text">Origin ▸ destination.</span>
        {{ " " }}<span class="type-label-md text-text">FA</span> means the player was a free agent;
        <span class="type-label-md text-text">OUT</span> means released or retired. Select a team
        code to open the team, or a name to open the player.
      </p>
      <p v-for="help in STATUS_HELP" :key="help.status" class="flex items-center gap-3">
        <StatusPill :status="help.status" :reliability="help.reliability" />
        <span>{{ help.text }}</span>
      </p>
    </div>
  </details>
</template>
