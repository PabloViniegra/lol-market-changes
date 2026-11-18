<script setup lang="ts">
import { useMeta } from "../api/meta";

const { data: meta, isPending, isError } = useMeta();
</script>

<template>
  <h1 class="text-2xl font-semibold">LoL Market Changes</h1>
  <p class="mt-2 text-neutral-400">
    <span v-if="isPending">Checking sync status…</span>
    <span v-else-if="isError">Sync status unavailable.</span>
    <span v-else-if="meta?.lastRunAt"
      >Last sync: {{ new Date(meta.lastRunAt).toLocaleString() }}</span
    >
    <span v-else>No sync has run yet.</span>
  </p>
</template>
