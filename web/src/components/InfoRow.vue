<script setup lang="ts">
import BoardFlap from "./BoardFlap.vue";

defineProps<{
  code: string;
  from?: string;
  title: string;
  detail: string;
  href: string;
  titleClass?: string;
}>();
</script>

<template>
  <li class="board-row relative hover:bg-surface-raised">
    <div class="flex shrink-0 items-center gap-2">
      <template v-if="from">
        <BoardFlap :code="from" placeholder="" :team="null" :empty-label="from" />
        <span class="text-text-muted" aria-hidden="true">▸</span>
        <span class="sr-only">to</span>
      </template>
      <BoardFlap :code="code" placeholder="" :team="null" :empty-label="code" />
    </div>
    <div class="min-w-0 flex-1" :class="{ 'max-md:basis-full': from }">
      <a :href="href" class="row-link block type-board-player hover:text-primary">
        <span :class="titleClass">{{ title }}</span>
      </a>
      <p class="type-body-sm text-text-muted">{{ detail }}</p>
    </div>
  </li>
</template>
