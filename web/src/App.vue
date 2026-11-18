<script setup lang="ts">
import { computed } from "vue";
import { useMeta } from "./api/meta";
import { useNow } from "./composables/useNow";
import { formatAge, minutesAgo } from "./format";

const STALE_AFTER_MINUTES = 60;

const { data: meta } = useMeta();
const now = useNow();

const age = computed(() =>
  meta.value?.lastRunAt ? minutesAgo(meta.value.lastRunAt, now.value) : null,
);

const freshness = computed(() => {
  if (!meta.value) return null;

  if (age.value === null) return "Not synced yet";

  return `Updated ${formatAge(age.value)}`;
});

const staleHours = computed(() =>
  age.value !== null && age.value > STALE_AFTER_MINUTES ? Math.floor(age.value / 60) : null,
);
</script>

<template>
  <div class="mx-auto flex min-h-dvh max-w-content flex-col px-4 md:px-6">
    <header class="flex flex-wrap items-center gap-x-8 gap-y-2 border-b border-line py-4">
      <RouterLink to="/" class="flex items-center gap-3 type-heading-md hover:text-primary">
        <img src="/logo.webp" alt="" width="32" height="32" class="size-8" />
        Transfer Board
      </RouterLink>
      <nav aria-label="Main" class="flex gap-6 type-label-md">
        <RouterLink to="/" class="nav-link text-text-muted hover:text-text">Dashboard</RouterLink>
        <RouterLink to="/feed" class="nav-link text-text-muted hover:text-text">Feed</RouterLink>
      </nav>
      <p v-if="freshness" class="ml-auto type-body-sm text-text-muted">
        {{ freshness }}
      </p>
    </header>

    <div
      v-if="staleHours !== null"
      class="mt-4 rounded-panel bg-surface-raised px-6 py-3 type-body-sm"
      role="status"
    >
      Data hasn't refreshed for {{ staleHours }} {{ staleHours === 1 ? "hour" : "hours" }}. Moves
      from Leaguepedia may be missing.
    </div>

    <main class="flex-1 py-8 md:py-12">
      <RouterView />
    </main>

    <footer class="border-t border-line py-4 type-body-sm text-text-muted">
      <a v-if="meta" :href="meta.attribution.url" class="underline hover:text-text">
        {{ meta.attribution.text }}
      </a>
    </footer>
  </div>
</template>

<style scoped>
.nav-link.router-exact-active {
  color: var(--color-text);
  text-decoration: underline;
  text-decoration-color: var(--color-primary);
  text-decoration-thickness: 2px;
  text-underline-offset: 6px;
}
</style>
