<script setup lang="ts">
import { ref, watch } from "vue";
import { usePrefetchTeam } from "../api/teams";
import { isDarkLogo, measureLogo } from "../composables/useLogoTone";

const props = defineProps<{
  code: string | null;
  placeholder: string;
  team: string | null;
  logo?: string | null;
  emptyLabel?: string;
  flip?: boolean;
}>();

const failed = ref(false);
const prefetchTeam = usePrefetchTeam();

watch(
  () => props.logo,
  () => (failed.value = false),
);
</script>

<template>
  <component
    :is="team ? 'RouterLink' : 'span'"
    :to="team ? `/teams/${encodeURIComponent(team)}` : undefined"
    :title="team ?? undefined"
    @pointerenter="team && prefetchTeam(team)"
    @focus="team && prefetchTeam(team)"
    class="flap relative flex h-9 min-w-20 shrink-0 items-center justify-center rounded-flap bg-surface-raised px-2 type-board-code md:h-11 md:min-w-26"
    :class="[code ? 'text-text' : 'text-text-muted', { 'flap-flip': flip, 'hover:bg-line': team }]"
  >
    <span
      v-for="half in ['flap-top', 'flap-bottom', 'flap-sizer']"
      :key="half"
      :class="half"
      aria-hidden="true"
    >
      <img
        v-if="logo && !failed"
        :src="logo"
        alt=""
        width="24"
        height="24"
        loading="lazy"
        decoding="async"
        referrerpolicy="no-referrer"
        crossorigin="anonymous"
        class="size-5 shrink-0 object-contain md:size-6"
        :class="{ 'logo-plate': isDarkLogo(logo) }"
        @load="measureLogo(logo, $event.target as HTMLImageElement)"
        @error="failed = true"
      />
      {{ code ?? placeholder }}
    </span>
    <span class="sr-only">{{ team ?? emptyLabel }}</span>
  </component>
</template>

<style scoped>
.flap::after {
  content: "";
  position: absolute;
  inset-inline: 0;
  top: 50%;
  height: 1px;
  background-color: var(--color-line);
}

.flap-top,
.flap-bottom,
.flap-sizer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.flap-top,
.flap-bottom {
  position: absolute;
  inset: 0;
  transform-origin: center 50%;
}

.flap-top {
  clip-path: inset(0 0 50% 0);
}

.flap-bottom {
  clip-path: inset(50% 0 0 0);
}

.flap-sizer {
  visibility: hidden;
}

.logo-plate {
  padding: 2px;
  border-radius: var(--radius-flap);
  background-color: var(--color-text);
}

@media (prefers-reduced-motion: no-preference) {
  .flap-flip {
    --flap-half-duration: 140ms;
    --flap-delay: 0ms;
  }

  .flap-flip ~ .flap-flip {
    --flap-delay: 30ms;
  }

  .flap-flip .flap-top {
    animation: flap-top var(--flap-half-duration) var(--ease-out) var(--flap-delay);
  }

  .flap-flip .flap-bottom {
    animation: flap-bottom var(--flap-half-duration) var(--ease-out)
      calc(var(--flap-half-duration) + var(--flap-delay)) backwards;
  }
}

@keyframes flap-top {
  to {
    transform: perspective(200px) rotateX(-90deg);
  }
}

@keyframes flap-bottom {
  from {
    transform: perspective(200px) rotateX(90deg);
  }
}
</style>
