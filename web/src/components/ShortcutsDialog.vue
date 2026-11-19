<script setup lang="ts">
import { useTemplateRef } from "vue";
import { GO_TO } from "../composables/useShortcuts";

const dialog = useTemplateRef<HTMLDialogElement>("dialog");

const shortcuts = [
  { keys: ["j"], action: "Next move on the board" },
  { keys: ["k"], action: "Previous move on the board" },
  ...Object.entries(GO_TO).map(([key, { label }]) => ({
    keys: ["g", key],
    action: `Go to ${label}`,
  })),
  { keys: ["Ctrl", "/"], together: true, action: "Search players in the feed" },
  { keys: ["?"], action: "Show these shortcuts" },
];

const closeOnBackdrop = (event: MouseEvent) => {
  if (event.target === dialog.value) dialog.value?.close();
};

defineExpose({ open: () => dialog.value?.showModal() });
</script>

<template>
  <dialog
    ref="dialog"
    aria-labelledby="shortcuts-heading"
    class="m-auto w-full max-w-96 rounded-panel bg-surface-raised text-text backdrop:bg-background/60"
    @click="closeOnBackdrop"
  >
    <div class="flex items-center justify-between gap-4 border-b border-line px-6 py-3">
      <h2 id="shortcuts-heading" class="type-heading-md">Keyboard shortcuts</h2>
      <button
        type="button"
        class="h-10 rounded-control px-3 type-label-md text-text-muted hover:bg-line hover:text-text"
        @click="dialog?.close()"
      >
        Close
      </button>
    </div>
    <dl class="flex flex-col gap-3 px-6 py-4">
      <div
        v-for="item in shortcuts"
        :key="item.action"
        class="flex items-center justify-between gap-4"
      >
        <dt class="flex gap-1">
          <template v-for="(key, index) in item.keys" :key="key">
            <span v-if="index" class="sr-only">{{ "together" in item ? "plus" : "then" }}</span>
            <kbd class="min-w-7 rounded-flap bg-line px-2 py-1 text-center type-label-md">{{
              key
            }}</kbd>
          </template>
        </dt>
        <dd class="type-body-sm text-text-muted">{{ item.action }}</dd>
      </div>
    </dl>
  </dialog>
</template>
