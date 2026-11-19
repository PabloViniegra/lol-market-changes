<script setup lang="ts">
import type { FeedItem } from "@lol/shared";
import { computed } from "vue";
import { useNewIds } from "../composables/useNewIds";
import { useNow } from "../composables/useNow";
import BoardRow from "./BoardRow.vue";
import BoardRowSkeleton from "./BoardRowSkeleton.vue";

const props = withDefaults(
  defineProps<{
    items: FeedItem[] | undefined;
    pending: boolean;
    empty?: string;
    rows?: number;
    hidePlayer?: boolean;
  }>(),
  { rows: 6 },
);

const NAME_WIDTHS = ["w-28", "w-40", "w-24", "w-36", "w-32", "w-20"];

const now = useNow();
const fresh = useNewIds(computed(() => props.items?.map((item) => item.id)));
</script>

<template>
  <ul
    v-if="pending"
    class="flex flex-col gap-px overflow-hidden rounded-panel bg-line"
    aria-busy="true"
    aria-label="Loading moves"
  >
    <BoardRowSkeleton
      v-for="n in rows"
      :key="n"
      :name-width="NAME_WIDTHS[(n - 1) % NAME_WIDTHS.length]!"
      :hide-player="hidePlayer"
    />
  </ul>
  <div v-else-if="!items?.length && empty" class="rounded-panel bg-surface p-6 text-text-muted">
    <p class="max-w-prose wrap-anywhere">{{ empty }}</p>
  </div>
  <ul v-else-if="items?.length" class="flex flex-col gap-px overflow-hidden rounded-panel bg-line">
    <BoardRow
      v-for="item in items"
      :key="item.id"
      :item="item"
      :now="now"
      :fresh="fresh.has(item.id)"
      :hide-player="hidePlayer"
    />
  </ul>
</template>
