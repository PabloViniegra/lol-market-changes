<script setup lang="ts">
import { computed } from "vue";
import WarningIcon from "./WarningIcon.vue";

const props = defineProps<{
  id: string;
  label: string;
  error?: string;
  hint?: string;
  optional?: boolean;
}>();

const describedBy = computed(
  () =>
    [props.hint && `${props.id}-hint`, props.error && `${props.id}-error`]
      .filter(Boolean)
      .join(" ") || undefined,
);
</script>

<template>
  <div class="flex flex-col gap-1">
    <label :for="id" class="flex min-h-4 items-baseline gap-1 type-label-md">
      {{ label }}
      <span v-if="optional" class="type-body-sm text-text-muted">(optional)</span>
    </label>
    <p v-if="hint" :id="`${id}-hint`" class="type-body-sm text-text-muted">{{ hint }}</p>
    <slot :described-by="describedBy" :invalid="Boolean(error)" />
    <p v-if="error" :id="`${id}-error`" class="flex items-center gap-1.5 type-body-sm">
      <WarningIcon />
      {{ error }}
    </p>
  </div>
</template>
