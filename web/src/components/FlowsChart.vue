<script setup lang="ts">
import type { FlowsResponse } from "@lol/shared";
import { computed, onBeforeUnmount, onMounted, ref, shallowRef, useTemplateRef, watch } from "vue";
import { useTeams } from "../api/teams";
import { useMediaQuery } from "../composables/useMediaQuery";
import { LEAGUES } from "../format";

type Link = FlowsResponse["links"][number];

type Side = "from" | "to";

interface Node {
  key: string;
  side: Side;
  name: string;
  value: number;
  y: number;
  h: number;
}

interface Ribbon {
  key: string;
  link: Link;
  sy: number;
  ty: number;
  h: number;
}

interface Layout {
  height: number;
  nodes: Node[];
  ribbons: Ribbon[];
}

const props = defineProps<{ links: FlowsResponse["links"]; level: FlowsResponse["level"] }>();

const MAX_LINKS = 24;
const NODE_WIDTH = 12;
const NODE_GAP = 16;
const PADDING_Y = 16;
const MORPH_MS = 360;

const container = useTemplateRef("container");
const width = ref(0);
const observer = new ResizeObserver(([entry]) => (width.value = entry?.contentRect.width ?? 0));

onMounted(() => container.value && observer.observe(container.value));

const reducedMotion = useMediaQuery("(prefers-reduced-motion: reduce)");

const labelSpace = computed(() => Math.min(200, Math.max(96, Math.round(width.value * 0.24))));

const x1 = computed(() =>
  Math.max(labelSpace.value + NODE_WIDTH * 4, width.value - labelSpace.value),
);

const routeKey = (link: Link) => `${link.source}\u0000${link.target}`;

function stack(totals: Map<string, number>, side: Side, scale: number, height: number) {
  const values = [...totals.values()];

  const span =
    values.reduce((sum, value) => sum + value * scale, 0) + NODE_GAP * (values.length - 1);

  let y = (height - span) / 2;

  return [...totals].map(([name, value]) => {
    const node = { key: `${side}:${name}`, side, name, value, y, h: value * scale };
    y += node.h + NODE_GAP;

    return node;
  });
}

function layout(links: Link[]): Layout {
  const top = links.toSorted((a, b) => b.value - a.value).slice(0, MAX_LINKS);

  const totals = (pick: (link: Link) => string) => {
    const sums = new Map<string, number>();

    for (const link of top) sums.set(pick(link), (sums.get(pick(link)) ?? 0) + link.value);

    return new Map([...sums].toSorted((a, b) => b[1] - a[1]));
  };

  const origins = totals((link) => link.source);
  const destinations = totals((link) => link.target);
  const total = top.reduce((sum, link) => sum + link.value, 0);
  const rows = Math.max(origins.size, destinations.size);
  const height = Math.min(480, Math.max(176, rows * 44));

  const scale = Math.min(
    (height - PADDING_Y - NODE_GAP * (origins.size - 1)) / total,
    (height - PADDING_Y - NODE_GAP * (destinations.size - 1)) / total,
  );

  const nodes = [
    ...stack(origins, "from", scale, height),
    ...stack(destinations, "to", scale, height),
  ];

  const order = new Map(nodes.map((node, i) => [node.key, i]));
  const cursor = new Map(nodes.map((node) => [node.key, node.y]));
  const sy = new Map<Link, number>();
  const ty = new Map<Link, number>();

  // Ribbons leave and enter each node in the order of their far end, so they don't cross inside it.
  for (const [side, far, ends] of [
    ["from", "to", sy],
    ["to", "from", ty],
  ] as const) {
    const near = (link: Link) => `${side}:${side === "from" ? link.source : link.target}`;
    const other = (link: Link) => `${far}:${far === "from" ? link.source : link.target}`;

    for (const link of top.toSorted((a, b) => order.get(other(a))! - order.get(other(b))!)) {
      ends.set(link, cursor.get(near(link))!);
      cursor.set(near(link), cursor.get(near(link))! + link.value * scale);
    }
  }

  return {
    height,
    nodes,
    ribbons: top.map((link) => ({
      key: routeKey(link),
      link,
      sy: sy.get(link)!,
      ty: ty.get(link)!,
      h: link.value * scale,
    })),
  };
}

const goal = computed(() => layout(props.links));
const frame = shallowRef(goal.value);

// Elements that exist in both layouts morph in place; the rest pair up by rank so a level switch
// reflows the diagram instead of swapping it, and anything left over grows from its centre line.
function startOf<T extends { key: string }>(
  prev: T[],
  next: T[],
  collapsed: (item: T) => T,
  group: (item: T) => string = () => "",
) {
  const prevByKey = new Map(prev.map((item) => [item.key, item]));
  const nextKeys = new Set(next.map((item) => item.key));
  const spare = prev.filter((item) => !nextKeys.has(item.key));

  return next.map((item) => {
    const same = prevByKey.get(item.key);

    if (same) return same;
    const index = spare.findIndex((candidate) => group(candidate) === group(item));

    return index === -1 ? collapsed(item) : spare.splice(index, 1)[0]!;
  });
}

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

let raf = 0;

watch(goal, (next) => {
  cancelAnimationFrame(raf);

  if (reducedMotion.value) {
    frame.value = next;

    return;
  }

  const shown = frame.value;

  const nodeStart = startOf(
    shown.nodes,
    next.nodes,
    (node) => ({ ...node, y: node.y + node.h / 2, h: 0 }),
    (node) => node.side,
  );

  const ribbonStart = startOf(shown.ribbons, next.ribbons, (ribbon) => ({
    ...ribbon,
    sy: ribbon.sy + ribbon.h / 2,
    ty: ribbon.ty + ribbon.h / 2,
    h: 0,
  }));

  const startHeight = shown.height;
  const began = performance.now();

  const step = (now: number) => {
    const t = Math.min(1, (now - began) / MORPH_MS);
    const e = 1 - (1 - t) ** 4;
    frame.value = {
      ...next,
      height: lerp(startHeight, next.height, e),
      nodes: next.nodes.map((node, i) => ({
        ...node,
        y: lerp(nodeStart[i]!.y, node.y, e),
        h: lerp(nodeStart[i]!.h, node.h, e),
      })),
      ribbons: next.ribbons.map((ribbon, i) => ({
        ...ribbon,
        sy: lerp(ribbonStart[i]!.sy, ribbon.sy, e),
        ty: lerp(ribbonStart[i]!.ty, ribbon.ty, e),
        h: lerp(ribbonStart[i]!.h, ribbon.h, e),
      })),
    };

    if (t < 1) raf = requestAnimationFrame(step);
  };

  raf = requestAnimationFrame(step);
});

onBeforeUnmount(() => {
  observer.disconnect();
  cancelAnimationFrame(raf);
});

function ribbonPath({ sy, ty, h }: Ribbon) {
  const a = labelSpace.value + NODE_WIDTH;
  const b = x1.value;
  const mid = (a + b) / 2;
  const t = Math.max(h, 1.5);

  return `M${a},${sy}C${mid},${sy} ${mid},${ty} ${b},${ty}L${b},${ty + t}C${mid},${ty + t} ${mid},${sy + t} ${a},${sy + t}Z`;
}

const isLeague = (name: string) => LEAGUES.some((league) => league === name);

const teams = useTeams(() => props.level === "team");

const teamLeague = computed(
  () => new Map(teams.data.value?.teams.map((team) => [team.name, team.league]) ?? []),
);

function fill(name: string) {
  const league = isLeague(name) ? name : teamLeague.value.get(name);

  return league ? `var(--color-league-${league.toLowerCase()})` : "var(--color-text-muted)";
}

const moves = (count: number) => `${count} ${count === 1 ? "move" : "moves"}`;

function routeName({ source, target }: Link) {
  if (source !== target) return `${source} to ${target}`;

  return source === "Other" ? "Between teams outside the tracked leagues" : `Within ${source}`;
}

type Active = { kind: "route"; key: string } | { kind: "node"; key: string } | null;

const hovered = ref<Active>(null);
const focused = ref<Active>(null);

// A refetch can drop the hovered or focused route from the top links; ignore it instead of dimming everything.
const active = computed(
  () =>
    [hovered.value, focused.value].find(
      (candidate) =>
        candidate &&
        (candidate.kind === "route" ? frame.value.ribbons : frame.value.nodes).some(
          (item) => item.key === candidate.key,
        ),
    ) ?? null,
);

const activeRoute = computed(() =>
  active.value?.kind === "route"
    ? frame.value.ribbons.find((ribbon) => ribbon.key === active.value!.key)
    : undefined,
);

const activeNode = computed(() =>
  active.value?.kind === "node"
    ? frame.value.nodes.find((node) => node.key === active.value!.key)
    : undefined,
);

function ribbonLit(ribbon: Ribbon) {
  if (!active.value) return true;

  if (active.value.kind === "route") return active.value.key === ribbon.key;

  return (
    active.value.key === `from:${ribbon.link.source}` ||
    active.value.key === `to:${ribbon.link.target}`
  );
}

function nodeLit(node: Node) {
  if (!active.value) return true;

  return frame.value.ribbons.some(
    (ribbon) =>
      ribbonLit(ribbon) &&
      (node.key === `from:${ribbon.link.source}` || node.key === `to:${ribbon.link.target}`),
  );
}

const focusIndex = ref(0);
const ribbonEls = useTemplateRef<SVGPathElement[]>("ribbonEls");

watch(
  () => frame.value.ribbons.length,
  (count) => (focusIndex.value = Math.min(focusIndex.value, Math.max(0, count - 1))),
);

function onKeydown(event: KeyboardEvent, index: number) {
  const last = frame.value.ribbons.length - 1;

  const next =
    event.key === "ArrowDown" || event.key === "ArrowRight"
      ? Math.min(last, index + 1)
      : event.key === "ArrowUp" || event.key === "ArrowLeft"
        ? Math.max(0, index - 1)
        : event.key === "Home"
          ? 0
          : event.key === "End"
            ? last
            : null;

  if (next === null) return;
  event.preventDefault();
  focusIndex.value = next;
  ribbonEls.value?.[next]?.focus();
}

function onFocus(ribbon: Ribbon, index: number) {
  focusIndex.value = index;
  focused.value = { kind: "route", key: ribbon.key };
}
</script>

<template>
  <div>
    <p class="mb-3 flex min-h-6 flex-wrap items-baseline gap-x-2 type-body-md" aria-hidden="true">
      <template v-if="activeRoute">
        <template v-if="activeRoute.link.source === activeRoute.link.target">
          <span class="font-bold">{{ routeName(activeRoute.link) }}</span>
        </template>
        <span v-else class="font-bold">
          {{ activeRoute.link.source }}
          <span class="text-text-muted">▸</span>
          {{ activeRoute.link.target }}
        </span>
        <span class="type-data-tabular">{{ moves(activeRoute.link.value) }}</span>
      </template>
      <template v-else-if="activeNode">
        <span class="font-bold">{{ activeNode.name }}</span>
        <span class="type-data-tabular"
          >{{ moves(activeNode.value) }} {{ activeNode.side === "from" ? "out" : "in" }}</span
        >
      </template>
      <span v-else class="type-body-sm text-text-muted">
        Point at a route or tab through them to trace where players went.
      </span>
    </p>
    <div
      ref="container"
      class="relative"
      :style="{ height: `${frame.height}px` }"
      @pointerleave="hovered = null"
    >
      <svg
        v-if="width"
        role="group"
        aria-label="Flows diagram"
        :width="width"
        :height="frame.height"
        class="absolute inset-0 block"
      >
        <g role="list" aria-label="Routes, largest first">
          <path
            v-for="(ribbon, i) in frame.ribbons"
            ref="ribbonEls"
            :key="ribbon.key"
            role="listitem"
            :aria-label="`${routeName(ribbon.link)}, ${moves(ribbon.link.value)}`"
            :tabindex="i === focusIndex ? 0 : -1"
            :d="ribbonPath(ribbon)"
            :fill="fill(ribbon.link.source)"
            class="ribbon"
            :class="{ lit: active && ribbonLit(ribbon), dim: !ribbonLit(ribbon) }"
            @pointerenter="hovered = { kind: 'route', key: ribbon.key }"
            @focus="onFocus(ribbon, i)"
            @blur="focused = null"
            @keydown="onKeydown($event, i)"
          />
        </g>
        <rect
          v-for="node in frame.nodes"
          :key="node.key"
          :x="node.side === 'from' ? labelSpace : x1"
          :y="node.y"
          :width="NODE_WIDTH"
          :height="Math.max(node.h, 2)"
          rx="3"
          :fill="fill(node.name)"
          :opacity="nodeLit(node) ? 1 : 0.3"
          aria-hidden="true"
          @pointerenter="hovered = { kind: 'node', key: node.key }"
        />
      </svg>
      <div
        v-for="node in frame.nodes"
        :key="node.key"
        class="absolute flex -translate-y-1/2 items-baseline gap-1.5 whitespace-nowrap type-label-md"
        :class="[
          node.side === 'from' ? 'justify-end pr-2' : 'pl-2',
          nodeLit(node) ? 'text-text' : 'text-text-muted',
        ]"
        :style="{
          top: `${node.y + node.h / 2}px`,
          left: node.side === 'from' ? '0px' : `${x1 + NODE_WIDTH}px`,
          width: `${node.side === 'from' ? labelSpace : labelSpace - NODE_WIDTH}px`,
        }"
        aria-hidden="true"
        @pointerenter="hovered = { kind: 'node', key: node.key }"
      >
        <span class="min-w-0 truncate" :title="node.name">{{ node.name }}</span>
        <span class="tabular-nums text-text-muted"
          >{{ node.value }} {{ node.side === "from" ? "out" : "in" }}</span
        >
      </div>
    </div>
    <details v-if="level === 'team'" class="mt-3 md:hidden">
      <summary
        class="inline-flex min-h-11 cursor-pointer items-center rounded-control type-label-md"
      >
        View team routes
      </summary>
      <ul class="divide-y divide-line type-body-sm">
        <li
          v-for="ribbon in goal.ribbons"
          :key="ribbon.key"
          class="flex items-baseline justify-between gap-3 py-2"
        >
          <span class="min-w-0 wrap-anywhere">{{ routeName(ribbon.link) }}</span>
          <span class="shrink-0 text-text-muted tabular-nums">{{ moves(ribbon.link.value) }}</span>
        </li>
      </ul>
    </details>
  </div>
</template>

<style scoped>
.ribbon {
  opacity: 0.4;
  cursor: default;
  outline: none;
}

.ribbon.lit {
  opacity: 0.85;
}

.ribbon.dim {
  opacity: 0.1;
}

.ribbon:focus-visible {
  stroke: var(--color-primary);
  stroke-width: 2px;
  opacity: 0.85;
}
</style>
