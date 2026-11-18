<script setup lang="ts">
const WIDTHS = ["w-24", "w-32", "w-20", "w-28", "w-24"];
</script>

<template>
  <div class="rounded-panel bg-surface p-6" aria-hidden="true">
    <span class="block h-3 w-28 rounded-flap bg-surface-raised" />
    <span class="mt-2 mb-5 block h-3 w-48 max-w-full rounded-flap bg-surface-raised" />
    <ul class="flex flex-col divide-y divide-line">
      <li
        v-for="(width, i) in WIDTHS"
        :key="i"
        class="flex items-center justify-between gap-4 py-3"
      >
        <span class="h-4 rounded-flap bg-surface-raised" :class="width" />
        <span class="h-3 w-12 rounded-flap bg-surface-raised" />
      </li>
    </ul>
  </div>
</template>
