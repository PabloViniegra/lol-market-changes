<script setup lang="ts">
import type { FlowsResponse } from "@lol/shared";
import { SankeyChart } from "echarts/charts";
import { TooltipComponent } from "echarts/components";
import { use } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import { computed } from "vue";
import VChart from "vue-echarts";
import { LEAGUES } from "../format";

use([CanvasRenderer, SankeyChart, TooltipComponent]);

const props = defineProps<{ links: FlowsResponse["links"] }>();

const MAX_LINKS = 24;

const token = (name: string) =>
  getComputedStyle(document.documentElement).getPropertyValue(`--color-${name}`).trim();

const nodeColor = (name: string) =>
  LEAGUES.some((league) => league === name)
    ? token(`league-${name.toLowerCase()}`)
    : token("text-muted");

const label = (id: string) => id.slice(id.indexOf(":") + 1);

const topLinks = computed(() =>
  props.links.toSorted((a, b) => b.value - a.value).slice(0, MAX_LINKS),
);

// Origins and destinations get separate nodes so moves inside a league don't form a cycle.
const option = computed(() => {
  const names = new Set(
    topLinks.value.flatMap((link) => [`from:${link.source}`, `to:${link.target}`]),
  );

  return {
    textStyle: { fontFamily: getComputedStyle(document.body).fontFamily },
    tooltip: {
      trigger: "item",
      backgroundColor: token("surface-raised"),
      borderColor: token("line"),
      textStyle: { color: token("text"), fontSize: 13 },
      formatter: (params: {
        dataType: string;
        data: { source?: string; target?: string; value?: number };
        name: string;
        value: number;
      }) =>
        params.dataType === "edge"
          ? `${label(params.data.source ?? "")} ▸ ${label(params.data.target ?? "")}: ${params.data.value} moves`
          : `${label(params.name)}: ${params.value} moves`,
    },
    series: [
      {
        type: "sankey",
        left: 8,
        right: 200,
        top: 8,
        bottom: 8,
        nodeGap: 12,
        nodeWidth: 12,
        draggable: false,
        emphasis: { focus: "adjacency" },
        data: [...names].map((id) => ({
          name: id,
          itemStyle: { color: nodeColor(label(id)), borderWidth: 0 },
        })),
        links: topLinks.value.map((link) => ({
          source: `from:${link.source}`,
          target: `to:${link.target}`,
          value: link.value,
        })),
        lineStyle: { color: "source", opacity: 0.35, curveness: 0.5 },
        label: {
          color: token("text"),
          fontSize: 13,
          fontWeight: 700,
          formatter: (params: { name: string; value: number }) =>
            `${label(params.name)} ${params.value}`,
        },
      },
    ],
  };
});
</script>

<template>
  <div>
    <div class="h-80" aria-hidden="true">
      <VChart :option="option" autoresize />
    </div>
    <table class="sr-only">
      <caption>
        Moves from origin to destination
      </caption>
      <thead>
        <tr>
          <th scope="col">From</th>
          <th scope="col">To</th>
          <th scope="col">Moves</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="link in topLinks" :key="`${link.source}-${link.target}`">
          <td>{{ link.source }}</td>
          <td>{{ link.target }}</td>
          <td>{{ link.value }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
