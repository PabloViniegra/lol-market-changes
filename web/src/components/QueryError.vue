<script setup lang="ts">
defineProps<{ what: string }>();

defineEmits<{ retry: [] }>();
</script>

<template>
  <div class="flex flex-wrap items-center gap-4 rounded-panel bg-surface p-6" role="alert">
    <p>Couldn't load {{ what }}. Retrying in 30 seconds.</p>
    <button
      type="button"
      class="h-10 rounded-control bg-surface-raised px-4 type-label-md hover:bg-line"
      @click="$emit('retry')"
    >
      Retry now
    </button>
  </div>
</template>
