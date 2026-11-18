<script setup lang="ts">
import { useMeta } from "../api/meta";

const { data: meta, isPending, isError } = useMeta();
</script>

<template>
  <h1 class="type-heading-lg">Transfer Board</h1>
  <p class="mt-2 type-body-sm text-text-muted">
    <span v-if="isPending">Checking sync status…</span>
    <span v-else-if="isError">Sync status unavailable.</span>
    <span v-else-if="meta?.lastRunAt"
      >Last sync: {{ new Date(meta.lastRunAt).toLocaleString() }}</span
    >
    <span v-else>No sync has run yet.</span>
  </p>
</template>
