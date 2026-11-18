<script setup lang="ts">
import { useMeta } from "./api/meta";

const { data: meta } = useMeta();
</script>

<template>
  <div class="mx-auto flex min-h-dvh max-w-content flex-col px-6">
    <main class="flex-1 py-12">
      <RouterView />
    </main>
    <footer class="border-t border-line py-4 type-body-sm text-text-muted">
      <a v-if="meta" :href="meta.attribution.url" class="underline hover:text-text">
        {{ meta.attribution.text }}
      </a>
    </footer>
  </div>
</template>
