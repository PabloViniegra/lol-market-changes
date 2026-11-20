<script setup lang="ts">
import type { FeedStatus } from "@lol/shared";
import { STATUS_LABELS } from "../format";

defineProps<{ status: FeedStatus; reliability?: number | null }>();
</script>

<template>
  <span class="inline-flex items-center gap-2 whitespace-nowrap">
    <span
      class="inline-flex items-center gap-1.5 rounded-pill px-2.5 py-1 type-label-md"
      :class="{
        'pill-solid text-text': status === 'official',
        'bg-primary text-on-primary': status === 'confirmed',
        'pill-dashed bg-surface text-primary': status === 'rumor',
        'bg-surface text-text-muted line-through': status === 'denied',
        'bg-background text-text-muted': status === 'stale',
        'pill-muted-dashed bg-background text-text-muted': status === 'contradicted',
      }"
    >
      <span
        v-if="status === 'confirmed'"
        class="size-2 rounded-pill border border-dashed border-on-primary"
        aria-hidden="true"
      />
      {{ STATUS_LABELS[status] }}
    </span>
    <span
      v-if="status === 'rumor' && reliability"
      class="flex gap-0.5"
      role="img"
      :aria-label="`Reliability ${reliability} of 5`"
    >
      <span
        v-for="dot in 5"
        :key="dot"
        class="size-1.5 rounded-pill"
        :class="dot <= reliability ? 'bg-primary' : 'bg-line'"
      />
    </span>
  </span>
</template>

<style scoped>
.pill-solid {
  outline: 1px solid var(--color-text-muted);
  outline-offset: -1px;
}

.pill-dashed {
  outline: 1.5px dashed var(--color-primary);
  outline-offset: -1.5px;
}

.pill-muted-dashed {
  outline: 1px dashed var(--color-text-muted);
  outline-offset: -1px;
}
</style>
