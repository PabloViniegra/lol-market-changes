<script setup lang="ts">
import { useMeta } from "./api/meta";

const { data: meta } = useMeta();
</script>

<template>
  <div class="mx-auto flex min-h-dvh max-w-6xl flex-col px-4">
    <main class="flex-1 py-8">
      <RouterView />
    </main>
    <footer class="border-t border-neutral-800 py-4 text-sm text-neutral-400">
      <a v-if="meta" :href="meta.attribution.url" class="underline hover:text-neutral-200">
        {{ meta.attribution.text }}
      </a>
    </footer>
  </div>
</template>
