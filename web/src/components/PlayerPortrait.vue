<script setup lang="ts">
import { ref, watch } from "vue";
import RoleIcon from "./RoleIcon.vue";

const props = defineProps<{ src: string | null; role?: string | null; size: number }>();

const failed = ref(false);

watch(
  () => props.src,
  () => (failed.value = false),
);
</script>

<template>
  <span
    class="flex shrink-0 items-end justify-center overflow-hidden rounded-flap bg-surface-raised text-text-muted"
    :style="{ width: `${size}px`, height: `${size}px` }"
    aria-hidden="true"
  >
    <img
      v-if="src && !failed"
      :src="src"
      alt=""
      :width="size"
      :height="size"
      loading="lazy"
      decoding="async"
      referrerpolicy="no-referrer"
      class="size-full object-cover object-top"
      @error="failed = true"
    />
    <span v-else class="flex size-full items-center justify-center text-line [&_svg]:size-2/5">
      <RoleIcon :role="role ?? null" />
    </span>
  </span>
</template>
