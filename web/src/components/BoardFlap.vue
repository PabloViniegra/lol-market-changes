<script setup lang="ts">
defineProps<{
  code: string | null;
  placeholder: string;
  team: string | null;
  emptyLabel?: string;
  flip?: boolean;
}>();
</script>

<template>
  <component
    :is="team ? 'RouterLink' : 'span'"
    :to="team ? `/teams/${encodeURIComponent(team)}` : undefined"
    :title="team ?? undefined"
    class="flap relative flex h-9 min-w-14 shrink-0 items-center justify-center rounded-flap bg-surface-raised px-1 type-board-code md:h-11 md:min-w-18"
    :class="[code ? 'text-text' : 'text-text-muted', { 'flap-flip': flip, 'hover:bg-line': team }]"
  >
    <span class="flap-half flap-top" aria-hidden="true">{{ code ?? placeholder }}</span>
    <span class="flap-half flap-bottom" aria-hidden="true">{{ code ?? placeholder }}</span>
    <span class="invisible" aria-hidden="true">{{ code ?? placeholder }}</span>
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

.flap-half {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.flap-top {
  clip-path: inset(0 0 50% 0);
  transform-origin: center 50%;
}

.flap-bottom {
  clip-path: inset(50% 0 0 0);
  transform-origin: center 50%;
}

@media (prefers-reduced-motion: no-preference) {
  .flap-flip .flap-top {
    animation: flap-top 280ms ease-in;
  }

  .flap-flip .flap-bottom {
    animation: flap-bottom 280ms ease-out 280ms backwards;
  }
}

@keyframes flap-top {
  from {
    transform: perspective(200px) rotateX(90deg);
  }
}

@keyframes flap-bottom {
  from {
    transform: perspective(200px) rotateX(-90deg);
  }
}
</style>
