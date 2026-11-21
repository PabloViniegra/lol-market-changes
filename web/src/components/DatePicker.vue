<script setup lang="ts">
import { computed, nextTick, ref, useId, useTemplateRef } from "vue";
import { onClickOutside } from "../composables/onClickOutside";
import { localToday, shortDate } from "../format";

defineOptions({ inheritAttrs: false });

const props = defineProps<{
  min?: string | null;
  max?: string | null;
  clearable?: boolean;
  describedBy?: string;
}>();

const model = defineModel<string>({ default: "" });
const root = useTemplateRef<HTMLElement>("root");
const trigger = useTemplateRef<HTMLButtonElement>("trigger");
const panel = useTemplateRef<HTMLElement>("panel");
const id = useId();
const open = ref(false);
const active = ref("");
const draft = ref("");
const draftError = ref("");

const monthFormat = new Intl.DateTimeFormat("en", {
  month: "long",
  year: "numeric",
  timeZone: "UTC",
});

const dayFormat = new Intl.DateTimeFormat("en", { dateStyle: "full", timeZone: "UTC" });
const weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

function isoDay(date: Date) {
  const iso = date.toISOString();

  return iso.slice(0, iso.indexOf("T"));
}

function validDay(value?: string | null): value is string {
  if (!value || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;

  const date = new Date(value);

  return !Number.isNaN(date.getTime()) && isoDay(date) === value;
}

const selected = computed(() => (validDay(model.value) ? model.value : ""));
const min = computed(() => (validDay(props.min) ? props.min : "0001-01-01"));
const max = computed(() => (validDay(props.max) ? props.max : "9999-12-31"));
const display = computed(() => (selected.value ? shortDate(selected.value, true) : "Pick a date"));
const today = ref(localToday());
const monthStart = computed(() => `${active.value.slice(0, 7)}-01`);

function offsetDay(value: string, days: number) {
  const date = new Date(value);

  date.setUTCDate(date.getUTCDate() + days);

  return isoDay(date);
}

function offsetMonth(value: string, months: number) {
  const date = new Date(value);
  const day = date.getUTCDate();

  date.setUTCDate(1);
  date.setUTCMonth(date.getUTCMonth() + months + 1);
  date.setUTCDate(0);
  date.setUTCDate(Math.min(day, date.getUTCDate()));

  return isoDay(date);
}

function inRange(day: string) {
  return day >= min.value && day <= max.value;
}

function canMoveMonth(step: number) {
  const month = offsetMonth(monthStart.value, step).slice(0, 7);

  return month >= min.value.slice(0, 7) && month <= max.value.slice(0, 7);
}

const weeks = computed(() => {
  if (!active.value) return [];

  const leading = (new Date(monthStart.value).getUTCDay() + 6) % 7;
  const month = active.value.slice(0, 7);

  const days = Array.from({ length: 42 }, (_, index) =>
    offsetDay(monthStart.value, index - leading),
  );

  return Array.from({ length: 6 }, (_, index) =>
    days.slice(index * 7, index * 7 + 7).map((date) => (date.startsWith(month) ? date : "")),
  ).filter((week) => week.some(Boolean));
});

function focusDay() {
  void nextTick(() =>
    panel.value?.querySelector<HTMLButtonElement>('button[tabindex="0"]')?.focus(),
  );
}

function setActive(day: string) {
  const date = new Date(day);

  active.value =
    date < new Date(min.value) ? min.value : date > new Date(max.value) ? max.value : day;
  focusDay();
}

function show() {
  today.value = localToday();
  draft.value = selected.value;
  draftError.value = "";
  open.value = true;
  setActive(selected.value || today.value);
  void nextTick(() => panel.value?.scrollIntoView({ block: "nearest" }));
}

function close(restoreFocus = false) {
  open.value = false;

  if (restoreFocus) trigger.value?.focus();
}

function choose(day: string) {
  model.value = day;
  close(true);
}

function applyDraft() {
  const day = draft.value.trim();

  if ((!day && props.clearable) || (validDay(day) && inRange(day))) {
    choose(day);

    return;
  }

  draftError.value = validDay(day)
    ? "Choose a date within the allowed range."
    : "Enter a valid date in YYYY-MM-DD format.";
}

function onDayKeydown(event: KeyboardEvent) {
  const day = active.value;
  const weekday = (new Date(day).getUTCDay() + 6) % 7;

  const offset = new Map([
    ["ArrowLeft", -1],
    ["ArrowRight", 1],
    ["ArrowUp", -7],
    ["ArrowDown", 7],
    ["Home", -weekday],
    ["End", 6 - weekday],
  ]).get(event.key);

  if (offset !== undefined) {
    event.preventDefault();
    setActive(offsetDay(day, offset));
  } else if (event.key === "PageUp" || event.key === "PageDown") {
    event.preventDefault();
    const step = event.key === "PageUp" ? -1 : 1;

    setActive(offsetMonth(day, step * (event.shiftKey ? 12 : 1)));
  }
}

function onFocusout(event: FocusEvent) {
  if (event.relatedTarget instanceof Node && !root.value?.contains(event.relatedTarget)) close();
}

function onEscape(event: KeyboardEvent) {
  if (!open.value) return;

  event.preventDefault();
  event.stopPropagation();
  close(true);
}

onClickOutside(root, () => close());
</script>

<template>
  <div ref="root" class="relative" @focusout="onFocusout" @keydown.esc="onEscape">
    <button
      v-bind="$attrs"
      ref="trigger"
      type="button"
      aria-haspopup="dialog"
      :aria-expanded="open"
      :aria-controls="id"
      :aria-describedby="[describedBy, `${id}-value`].filter(Boolean).join(' ')"
      class="flex w-full items-center gap-2 rounded-control border-control-border px-3 text-left type-body-md text-text hover:bg-line aria-expanded:bg-line aria-invalid:border-text"
      @click="open ? close() : show()"
      @keydown.down.prevent="show()"
    >
      <span :id="`${id}-value`" :class="{ 'text-text-muted': !selected }">{{ display }}</span>
      <svg
        viewBox="0 0 20 20"
        class="ml-auto size-5 shrink-0 text-text-muted"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <rect x="3.5" y="5" width="13" height="12" rx="2" />
        <path d="M6.5 3.5v3m7-3v3M3.5 9h13" />
      </svg>
    </button>

    <div
      v-show="open"
      :id="id"
      ref="panel"
      role="dialog"
      :aria-labelledby="`${id}-month`"
      :inert="!open"
      class="menu-panel max-w-80 p-2 max-md:static"
    >
      <div v-if="active" class="flex items-center justify-between gap-2">
        <button
          type="button"
          aria-label="Previous month"
          class="calendar-nav"
          :disabled="!canMoveMonth(-1)"
          @click="setActive(offsetMonth(active, -1))"
        >
          <svg viewBox="0 0 20 20" class="size-5 rotate-180" fill="none" aria-hidden="true">
            <path d="m8 5 5 5-5 5" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
        <h2 :id="`${id}-month`" aria-live="polite" class="type-label-md text-text">
          {{ monthFormat.format(new Date(monthStart)) }}
        </h2>
        <button
          type="button"
          aria-label="Next month"
          class="calendar-nav"
          :disabled="!canMoveMonth(1)"
          @click="setActive(offsetMonth(active, 1))"
        >
          <svg viewBox="0 0 20 20" class="size-5" fill="none" aria-hidden="true">
            <path d="m8 5 5 5-5 5" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
      </div>
      <div v-if="active" role="grid" :aria-labelledby="`${id}-month`" @keydown="onDayKeydown">
        <div role="row" class="grid grid-cols-7 text-center type-body-sm text-text-muted">
          <span v-for="weekday in weekdays" :key="weekday" role="columnheader" class="py-1">
            {{ weekday }}
          </span>
        </div>
        <div v-for="(week, index) in weeks" :key="index" role="row" class="grid grid-cols-7">
          <div
            v-for="(day, column) in week"
            :key="column"
            role="gridcell"
            :aria-selected="day ? day === selected : undefined"
          >
            <button
              v-if="day"
              type="button"
              :aria-label="dayFormat.format(new Date(day))"
              :aria-current="day === today ? 'date' : undefined"
              :tabindex="day === active ? 0 : -1"
              :disabled="!inRange(day)"
              class="h-9 w-full rounded-flap type-body-sm enabled:cursor-pointer disabled:text-text-muted/60 aria-current:underline"
              :class="
                day === selected ? 'bg-text text-background' : 'text-text enabled:hover:bg-line'
              "
              @click="choose(day)"
            >
              {{ Number(day.slice(8)) }}
            </button>
          </div>
        </div>
      </div>
      <div class="mt-1 border-t border-line pt-2">
        <div class="flex gap-2">
          <input
            :id="`${id}-draft`"
            v-model="draft"
            type="text"
            aria-label="Enter date (YYYY-MM-DD)"
            :aria-invalid="draftError ? true : undefined"
            :aria-describedby="draftError ? `${id}-error` : undefined"
            placeholder="YYYY-MM-DD"
            autocomplete="off"
            spellcheck="false"
            class="h-9 min-w-0 flex-1 rounded-control border border-control-border bg-surface px-2 type-body-sm text-text placeholder:text-text-muted aria-invalid:border-text"
            @input="draftError = ''"
            @keydown.enter.prevent="applyDraft"
          />
          <button
            type="button"
            class="h-9 rounded-control bg-line px-2 type-label-md text-text hover:bg-surface"
            @click="applyDraft"
          >
            Apply
          </button>
        </div>
        <p v-if="draftError" :id="`${id}-error`" role="alert" class="mt-1 type-body-sm text-text">
          {{ draftError }}
        </p>
      </div>
      <button
        v-if="clearable && selected"
        type="button"
        class="mt-1 h-9 w-full rounded-flap px-2 text-left type-body-sm text-text-muted hover:bg-line hover:text-text"
        @click="choose('')"
      >
        Clear date
      </button>
    </div>
  </div>
</template>

<style scoped>
@reference "../style.css";

.calendar-nav {
  @apply grid size-9 place-items-center rounded-flap text-text-muted enabled:cursor-pointer enabled:hover:bg-line enabled:hover:text-text disabled:text-text-muted/60;
}
</style>
