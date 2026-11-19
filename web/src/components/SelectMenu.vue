<script setup lang="ts" generic="T extends string | number">
import { computed, nextTick, ref, useId } from "vue";
import { onClickOutside } from "../composables/onClickOutside";

defineOptions({ inheritAttrs: false });

const props = defineProps<{ options: readonly { value: T; label: string }[] }>();
const model = defineModel<T>({ required: true });

const open = ref(false);
const active = ref(0);
const root = ref<HTMLElement | null>(null);
const listId = useId();

const selectedIndex = computed(() =>
  Math.max(
    0,
    props.options.findIndex((option) => option.value === model.value),
  ),
);

const selected = computed(() => props.options[selectedIndex.value]);
const optionId = (index: number) => `${listId}-${index}`;

let typed = "";
let typedAt = 0;

function show(index = selectedIndex.value) {
  active.value = index;
  open.value = true;
  void nextTick(reveal);
}

function choose(index: number) {
  const option = props.options[index];

  if (option) model.value = option.value;
  open.value = false;
}

function reveal() {
  document.getElementById(optionId(active.value))?.scrollIntoView({ block: "nearest" });
}

function move(index: number) {
  active.value = Math.min(props.options.length - 1, Math.max(0, index));
  reveal();
}

function typeahead(key: string) {
  const now = Date.now();

  typed = now - typedAt > 600 ? key : typed + key;
  typedAt = now;

  const start = open.value ? active.value : selectedIndex.value;
  const order = [...props.options.keys()].map((i) => (start + 1 + i) % props.options.length);

  const match = order.find((i) =>
    props.options[i]?.label.toLowerCase().startsWith(typed.toLowerCase()),
  );

  if (match === undefined) return;

  if (open.value) move(match);
  else choose(match);
}

function onKeydown(event: KeyboardEvent) {
  const { key } = event;

  if (!open.value) {
    if (["ArrowDown", "ArrowUp", "Enter", " "].includes(key)) {
      event.preventDefault();
      show();
    } else if (key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
      typeahead(key);
    }

    return;
  }

  const target = {
    ArrowDown: active.value + 1,
    ArrowUp: active.value - 1,
    Home: 0,
    End: props.options.length - 1,
    PageDown: active.value + 5,
    PageUp: active.value - 5,
  }[key];

  if (target !== undefined) {
    event.preventDefault();
    move(target);
  } else if (key === "Enter" || key === " ") {
    event.preventDefault();
    choose(active.value);
  } else if (key === "Escape") {
    event.preventDefault();
    open.value = false;
  } else if (key === "Tab") {
    choose(active.value);
  } else if (key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
    typeahead(key);
  }
}

onClickOutside(root, () => (open.value = false));
</script>

<template>
  <div ref="root" class="relative">
    <button
      v-bind="$attrs"
      type="button"
      role="combobox"
      aria-haspopup="listbox"
      :aria-expanded="open"
      :aria-controls="listId"
      :aria-activedescendant="open ? optionId(active) : undefined"
      class="group flex h-10 w-full items-center gap-2 rounded-control border-2 border-control-border bg-surface-raised px-3 text-left type-body-md text-text hover:bg-line aria-expanded:bg-line aria-invalid:border-text"
      @click="open ? (open = false) : show()"
      @keydown="onKeydown"
    >
      <slot name="option" :option="selected" :selected="true">
        <span class="truncate">{{ selected?.label }}</span>
      </slot>
      <svg
        viewBox="0 0 20 20"
        class="ml-auto size-5 shrink-0 text-text-muted menu-chevron"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="m6 8 4 4 4-4" />
      </svg>
    </button>

    <ul v-show="open" :id="listId" role="listbox" @pointerdown.prevent class="menu-panel max-h-96">
      <li
        v-for="(option, index) in options"
        :id="optionId(index)"
        :key="String(option.value)"
        role="option"
        :aria-selected="option.value === model"
        class="menu-option gap-2 type-body-md"
        :class="index === active ? 'bg-line text-text' : 'text-text-muted'"
        @pointermove="active = index"
        @click="choose(index)"
      >
        <slot name="option" :option="option" :selected="option.value === model">
          <span class="truncate">{{ option.label }}</span>
        </slot>
        <svg
          v-if="option.value === model"
          viewBox="0 0 20 20"
          class="ml-auto size-5 shrink-0 text-primary"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="m5 10.5 3 3 7-7" />
        </svg>
      </li>
    </ul>
  </div>
</template>
