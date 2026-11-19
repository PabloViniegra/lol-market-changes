<script setup lang="ts">
import FlapSkeleton from "./FlapSkeleton.vue";

defineProps<{ nameWidth: string; hidePlayer?: boolean }>();
</script>

<template>
  <li
    class="flex flex-wrap items-center gap-x-4 gap-y-2 bg-surface p-3 md:min-h-16 md:flex-nowrap"
    aria-hidden="true"
  >
    <div class="flex shrink-0 items-center gap-2">
      <FlapSkeleton />
      <span class="text-line">▸</span>
      <FlapSkeleton />
    </div>
    <div class="min-w-0 flex-1">
      <span
        v-if="!hidePlayer"
        class="mb-1 block h-5 max-w-full rounded-flap bg-surface-raised"
        :class="nameWidth"
      />
      <span class="block h-3 w-32 max-w-full rounded-flap bg-surface-raised" />
    </div>
    <div class="flex w-full items-center gap-3 md:w-auto md:shrink-0 md:justify-end">
      <span class="h-3 w-16 rounded-flap bg-surface-raised" />
      <span class="h-6 w-20 rounded-pill bg-surface-raised" />
      <span class="h-3 w-12 rounded-flap bg-surface-raised md:w-24" />
    </div>
  </li>
</template>
