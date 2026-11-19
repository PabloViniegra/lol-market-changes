<script setup lang="ts">
defineProps<{ change: "joined" | "left" }>();
</script>

<template>
  <span class="type-label-md" :class="change === 'joined' ? 'text-primary' : 'text-text-muted'">
    {{ change === "joined" ? "Joined" : "Left" }}
  </span>
</template>
