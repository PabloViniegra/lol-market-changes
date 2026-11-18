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
    <header class="flex flex-wrap items-center gap-x-8 gap-y-1 border-b border-line py-2">
      <RouterLink to="/" class="flex items-center gap-3 type-heading-md hover:text-primary">
        <img src="/logo.webp" alt="" width="32" height="32" class="size-8" />
        Transfer Board
      </RouterLink>
      <nav aria-label="Main" class="flex gap-6 type-label-md">
        <RouterLink to="/" class="nav-link py-3 text-text-muted hover:text-text"
          >Dashboard</RouterLink
        >
        <RouterLink to="/feed" class="nav-link py-3 text-text-muted hover:text-text"
          >Feed</RouterLink
        >
        <RouterLink to="/rumors" class="nav-link py-3 text-text-muted hover:text-text"
          >Rumors</RouterLink
        >
      </nav>
      <p v-if="freshness && staleHours === null" class="type-body-sm text-text-muted md:ml-auto">
        {{ freshness }}
      </p>
    </header>

    <div
      v-if="staleHours !== null"
      class="mt-4 flex items-start gap-3 rounded-panel bg-surface-raised px-6 py-3 type-body-sm"
      role="status"
    >
      <svg
        viewBox="0 0 20 20"
        class="size-5 shrink-0 text-primary"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M10 3 2.5 16.5h15L10 3ZM10 8.5v3.5M10 14.5v.01" />
      </svg>
      <p>
        <span class="type-label-md"
          >Data hasn't refreshed for {{ staleHours }}
          {{ staleHours === 1 ? "hour" : "hours" }}.</span
        >
        Moves from Leaguepedia may be missing.
      </p>
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
