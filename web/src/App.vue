<script setup lang="ts">
import { computed } from "vue";
import { useMeta } from "./api/meta";
import LolLogo from "./components/LolLogo.vue";
import { useNow } from "./composables/useNow";
import { formatAge, minutesAgo } from "./format";

const STALE_AFTER_MINUTES = 60;

const { data: meta, isError: metaUnavailable } = useMeta();
const now = useNow();

const attribution = computed(
  () =>
    meta.value?.attribution ?? {
      text: "Official roster data from Leaguepedia, licensed CC BY-SA",
      url: "https://lol.fandom.com/",
    },
);

const age = computed(() => {
  const lastSuccess =
    meta.value?.lastSuccessfulRunAt ??
    (meta.value?.lastRunStatus === "ok" ? meta.value.lastRunAt : null);

  return lastSuccess ? minutesAgo(lastSuccess, now.value) : null;
});

const failedRefresh = computed(() => meta.value?.lastRunStatus === "error");

const warning = computed(() => {
  if (metaUnavailable.value) return "Couldn’t verify when data last refreshed.";

  if (failedRefresh.value) return "The latest data refresh failed.";

  if (age.value !== null && age.value > STALE_AFTER_MINUTES) {
    const hours = Math.floor(age.value / 60);

    return `Data hasn’t refreshed for ${hours} ${hours === 1 ? "hour" : "hours"}.`;
  }

  return null;
});

const freshness = computed(() =>
  age.value === null ? "Not synced yet" : `Updated ${formatAge(age.value)}`,
);
</script>

<template>
  <div class="mx-auto flex min-h-dvh max-w-content flex-col px-4 md:px-6">
    <header class="flex flex-wrap items-center gap-x-8 border-b border-line pt-3 md:pt-0">
      <RouterLink
        to="/"
        class="mr-auto flex items-center gap-3 type-heading-md hover:text-text-muted md:mr-0"
      >
        <img src="/logo.webp" alt="" width="32" height="32" class="size-8" />
        Transfer Board
      </RouterLink>
      <nav
        aria-label="Main"
        class="order-last mt-1 flex basis-full gap-6 type-label-md md:order-none md:mt-0 md:basis-auto"
      >
        <RouterLink to="/" class="nav-link py-3 text-text-muted hover:text-text md:py-4"
          >Dashboard</RouterLink
        >
        <RouterLink to="/feed" class="nav-link py-3 text-text-muted hover:text-text md:py-4"
          >Feed</RouterLink
        >
        <RouterLink to="/rumors" class="nav-link py-3 text-text-muted hover:text-text md:py-4"
          >Rumors</RouterLink
        >
      </nav>
      <p v-if="meta && !warning" class="type-body-sm text-text-muted md:ml-auto">
        {{ freshness }}
      </p>
      <LolLogo
        class="h-6 w-auto shrink-0 text-text-muted"
        :class="{ 'md:ml-auto': !meta || warning }"
      />
    </header>

    <div
      v-if="warning"
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
        <span class="type-label-md">{{ warning }}</span>
        <template v-if="(failedRefresh || metaUnavailable) && age !== null">
          Last successful refresh {{ formatAge(age) }}.
        </template>
        Moves from Leaguepedia may be missing.
      </p>
    </div>

    <main class="flex-1 pt-6 pb-8 md:pt-8 md:pb-12">
      <RouterView />
    </main>

    <footer class="flex flex-col gap-2 border-t border-line py-4 type-body-sm text-text-muted">
      <a :href="attribution.url" class="self-start underline hover:text-text">
        {{ attribution.text }}
      </a>
      <p>
        Transfer Board isn’t endorsed by Riot Games and doesn’t reflect the views or opinions of
        Riot Games or anyone officially involved in producing or managing League of Legends. League
        of Legends and Riot Games are trademarks or registered trademarks of Riot Games, Inc.
      </p>
    </footer>
  </div>
</template>

<style scoped>
.nav-link {
  margin-bottom: -1px;
  border-bottom: 2px solid transparent;
}

.nav-link.router-link-exact-active {
  color: var(--color-text);
  border-color: var(--color-text-muted);
}
</style>
