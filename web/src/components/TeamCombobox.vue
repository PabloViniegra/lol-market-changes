<script setup lang="ts">
import type { TeamsResponse } from "@lol/shared";
import { computed, nextTick, ref, useId } from "vue";
import { findTeam } from "../api/teams";
import { onClickOutside } from "../composables/onClickOutside";
import { teamCode } from "../format";
import LeagueTag from "./LeagueTag.vue";

defineOptions({ inheritAttrs: false });

const props = withDefaults(defineProps<{ teams: TeamsResponse["teams"]; fieldClass?: string }>(), {
  fieldClass: "h-10 bg-surface-raised",
});

const emit = defineEmits<{ select: [name: string] }>();
const model = defineModel<string>({ required: true });

const open = ref(false);
const active = ref(-1);
const root = ref<HTMLElement | null>(null);
const listId = useId();
const optionId = (index: number) => `${listId}-${index}`;

const matches = computed(() => {
  const key = model.value.trim().toLowerCase();

  const hits = key
    ? props.teams.filter(
        (team) =>
          team.name.toLowerCase().includes(key) || team.short?.toLowerCase().startsWith(key),
      )
    : props.teams;

  return hits
    .toSorted((a, b) => a.league.localeCompare(b.league) || a.name.localeCompare(b.name))
    .slice(0, 40);
});

const known = computed(() => findTeam(props.teams, model.value));

const visible = computed(
  () => open.value && matches.value.length > 0 && known.value?.name !== model.value,
);

function show() {
  open.value = true;
  active.value = -1;
}

function choose(index: number) {
  const team = matches.value[index];

  if (team) {
    model.value = team.name;
    emit("select", team.name);
  }

  open.value = false;
}

function move(delta: number) {
  if (!visible.value) return show();

  const last = matches.value.length - 1;

  active.value =
    active.value < 0 && delta < 0 ? last : Math.min(last, Math.max(0, active.value + delta));
  void nextTick(() =>
    document.getElementById(optionId(active.value))?.scrollIntoView({ block: "nearest" }),
  );
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    move(event.key === "ArrowDown" ? 1 : -1);
  } else if (event.key === "Enter" && visible.value && active.value >= 0) {
    event.preventDefault();
    choose(active.value);
  } else if (event.key === "Escape" && visible.value) {
    event.preventDefault();
    open.value = false;
  } else if (event.key === "Tab") {
    open.value = false;
  }
}

onClickOutside(root, () => (open.value = false));
</script>

<template>
  <div ref="root" class="relative">
    <input
      v-bind="$attrs"
      v-model="model"
      role="combobox"
      aria-autocomplete="list"
      :aria-expanded="visible"
      :aria-controls="listId"
      :aria-activedescendant="visible && active >= 0 ? optionId(active) : undefined"
      autocomplete="off"
      spellcheck="false"
      class="w-full rounded-control border-2 border-control-border px-3 type-body-md text-text placeholder:text-text-muted aria-invalid:border-text"
      :class="[fieldClass, { 'pr-24': known }]"
      @input="show"
      @click="show"
      @keydown="onKeydown"
    />
    <span
      v-if="known"
      class="pointer-events-none absolute inset-y-0 right-3 flex items-center gap-2 type-label-md text-text-muted"
      aria-hidden="true"
    >
      {{ teamCode(known.name, known.short) }}
      <LeagueTag :league="known.league" />
    </span>

    <ul
      v-show="visible"
      :id="listId"
      role="listbox"
      @pointerdown.prevent
      :aria-label="`Teams matching ${model || 'any name'}`"
      class="menu-panel max-h-72"
    >
      <li
        v-for="(team, index) in matches"
        :id="optionId(index)"
        :key="team.name"
        role="option"
        :aria-selected="index === active"
        class="menu-option gap-3"
        :class="index === active ? 'bg-line text-text' : 'text-text-muted'"
        @pointermove="active = index"
        @click="choose(index)"
      >
        <span class="w-12 shrink-0 type-label-md text-text">
          {{ teamCode(team.name, team.short) }}
        </span>
        <span class="truncate type-body-md">{{ team.name }}</span>
        <LeagueTag class="ml-auto shrink-0" :league="team.league" />
      </li>
    </ul>
  </div>
</template>
