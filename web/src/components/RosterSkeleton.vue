<script setup lang="ts">
const WIDTHS = ["w-24", "w-32", "w-20", "w-28", "w-24"];
</script>

<template>
  <div class="rounded-panel bg-surface px-3 md:px-6" aria-hidden="true">
    <div class="grid grid-cols-3 gap-4 border-b border-line py-4">
      <span class="h-3 w-10 rounded-flap bg-surface-raised" />
      <span class="h-3 w-24 max-w-full rounded-flap bg-surface-raised" />
      <span class="h-3 w-24 max-w-full rounded-flap bg-surface-raised" />
    </div>
    <ul class="flex flex-col divide-y divide-line">
      <li v-for="(width, i) in WIDTHS" :key="i" class="grid grid-cols-3 items-center gap-4 py-3">
        <span class="h-3 w-12 rounded-flap bg-surface-raised" />
        <span class="h-4 max-w-full rounded-flap bg-surface-raised" :class="width" />
        <span class="h-4 w-20 max-w-full rounded-flap bg-surface-raised" />
      </li>
    </ul>
  </div>
</template>
