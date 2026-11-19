<script setup lang="ts">
import FlapSkeleton from "./FlapSkeleton.vue";

defineProps<{ nameWidth: string; hidePlayer?: boolean }>();
</script>

<template>
  <li class="board-row" aria-hidden="true">
    <div class="flex shrink-0 items-center gap-2 md:min-w-60">
      <FlapSkeleton />
      <span class="text-line">▸</span>
      <FlapSkeleton />
    </div>
    <span
      v-if="!hidePlayer"
      class="size-12 shrink-0 rounded-flap bg-surface-raised max-md:hidden"
    />
    <div class="min-w-0 flex-1">
      <span
        v-if="!hidePlayer"
        class="mb-1 block h-5 max-w-full rounded-flap bg-surface-raised"
        :class="nameWidth"
      />
      <span class="block h-3 w-32 max-w-full rounded-flap bg-surface-raised" />
    </div>
    <div class="flex h-8 w-full items-center gap-3 md:w-auto md:shrink-0 md:justify-end">
      <span class="h-3 w-16 rounded-flap bg-surface-raised" />
      <span class="h-6 w-20 rounded-pill bg-surface-raised" />
      <span class="ml-auto h-3 w-12 rounded-flap bg-surface-raised md:ml-0 md:w-24" />
      <span class="hidden size-8 md:block" />
    </div>
  </li>
</template>
