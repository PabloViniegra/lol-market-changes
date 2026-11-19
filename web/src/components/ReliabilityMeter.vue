<script setup lang="ts">
import { useId } from "vue";

defineProps<{ labels: readonly string[]; describedBy?: string }>();

const model = defineModel<number>({ required: true });
const name = useId();
</script>

<template>
  <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
    <div class="flex gap-1" role="radiogroup" :aria-describedby="describedBy">
      <label
        v-for="(label, index) in labels"
        :key="label"
        class="group relative flex size-10 cursor-pointer items-center justify-center rounded-control bg-surface-raised hover:bg-line has-focus-visible:outline-2 has-focus-visible:outline-offset-2 has-focus-visible:outline-primary"
      >
        <input
          v-model="model"
          type="radio"
          :name="name"
          :value="index + 1"
          class="peer sr-only"
          :aria-label="`${index + 1} · ${label}`"
        />
        <span
          class="meter-dot size-2.5 rounded-pill"
          :class="index < model ? 'bg-primary' : 'bg-line group-hover:bg-text-muted'"
          :style="{ '--meter-delay': `${index * 25}ms` }"
          aria-hidden="true"
        />
      </label>
    </div>
    <p class="type-label-md text-text" aria-hidden="true">{{ model }} · {{ labels[model - 1] }}</p>
  </div>
</template>

<style scoped>
.meter-dot {
  transition:
    background-color 120ms var(--ease-out) var(--meter-delay),
    transform 160ms var(--ease-out) var(--meter-delay);
}

input:checked + .meter-dot {
  transform: scale(1.4);
}

@media (prefers-reduced-motion: reduce) {
  .meter-dot {
    transition: background-color 120ms linear;
  }

  input:checked + .meter-dot {
    transform: none;
  }
}
</style>
