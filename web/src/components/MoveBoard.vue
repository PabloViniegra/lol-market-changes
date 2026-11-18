<script setup lang="ts">
import type { FeedItem } from "@lol/shared";
import { computed } from "vue";
import { useNewIds } from "../composables/useNewIds";
import { useNow } from "../composables/useNow";
import BoardRow from "./BoardRow.vue";

const props = defineProps<{ items: FeedItem[] | undefined; pending: boolean; empty: string }>();

const now = useNow();
const fresh = useNewIds(computed(() => props.items?.map((item) => item.id)));
</script>

<template>
  <ul v-if="pending" class="flex flex-col gap-px bg-line" aria-busy="true" aria-label="Loading">
    <li v-for="n in 6" :key="n" class="h-16 bg-surface" />
  </ul>
  <p v-else-if="!items?.length" class="max-w-prose rounded-panel bg-surface p-6 text-text-muted">
    {{ empty }}
  </p>
  <ul v-else class="flex flex-col gap-px overflow-hidden rounded-panel bg-line">
    <BoardRow
      v-for="item in items"
      :key="item.id"
      :item="item"
      :now="now"
      :fresh="fresh.has(item.id)"
    />
  </ul>
</template>
