<script setup lang="ts">
defineProps<{ active: boolean }>();
</script>

<template>
  <button
    type="button"
    :aria-pressed="active"
    class="h-11 rounded-control px-3 type-label-md md:h-8"
    :class="active ? 'bg-text text-background' : 'bg-surface text-text-muted hover:text-text'"
  >
    <slot />
  </button>
</template>
