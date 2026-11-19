<script setup lang="ts">
import { computed } from "vue";
import { ROLES } from "../format";

const props = defineProps<{ role: string | null }>();

const LANES = {
  Top: "M3 3h10l-3.5 3H6v3.5L3 13Z",
  Mid: "M13.5 3H17v3.5L6.5 17H3v-3.5Z",
  Bot: "M17 17H7l3.5-3H14v-3.5l3-3.5Z",
  Jungle:
    "M10 17C7.6 13 7.8 7.6 11.2 3c-.6 4.6.6 9.4-1.2 14ZM8.6 16.6C5.6 15 3.8 11.6 3.5 7.5c2 2 4 5 5.1 9.1ZM11.4 16.6c3-1.6 4.8-5 5.1-9.1-2 2-4 5-5.1 9.1Z",
  Support: "M10 4l2.6 4-2.6 8.5L7.4 8ZM6.6 7.5H3l2.9 4.5h2ZM13.4 7.5H17l-2.9 4.5h-2Z",
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
