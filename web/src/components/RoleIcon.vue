<script setup lang="ts">
import { computed } from "vue";
import { ROLES } from "../format";

const props = defineProps<{ role: string | null }>();

const LANES = {
  Top: "M3 3h10l-3.5 3H6v3.5L3 13Z",
  Mid: "M13.5 3H17v3.5L6.5 17H3v-3.5Z",
  Bot: "M17 17H7l3.5-3H14v-3.5l3-3.5Z",
  Jungle:
    "M5.5 3.5c.4 5 2 9.4 4.5 13 2.5-3.6 4.1-8 4.5-13-1.2 2.6-2.7 4.6-4.5 6-1.8-1.4-3.3-3.4-4.5-6Z",
  Support: "M10 3.5 15.5 6v4c0 3.2-2.2 5.5-5.5 6.5-3.3-1-5.5-3.3-5.5-6.5V6Z",
} as const satisfies Record<(typeof ROLES)[number], string>;

const path = computed(() => {
  const role = ROLES.find((item) => item === props.role);

  return role ? LANES[role] : null;
});

const framed = computed(() => props.role === "Top" || props.role === "Mid" || props.role === "Bot");
</script>

<template>
  <svg v-if="path" viewBox="0 0 20 20" class="size-5 shrink-0" aria-hidden="true">
    <rect
      v-if="framed"
      x="3.75"
      y="3.75"
      width="12.5"
      height="12.5"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      opacity="0.35"
    />
    <path :d="path" fill="currentColor" />
  </svg>
</template>
