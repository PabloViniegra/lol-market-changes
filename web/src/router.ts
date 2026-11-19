import { createRouter, createWebHistory } from "vue-router";
import DashboardView from "./views/DashboardView.vue";

declare module "vue-router" {
  interface RouteMeta {
    title?: string;
    description?: string;
  }
}

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", component: DashboardView, meta: { title: "League of Legends Esports Transfers" } },
    {
      path: "/feed",
      component: () => import("./views/FeedView.vue"),
      meta: {
        title: "Market feed",
        description:
          "Browse and filter League of Legends esports roster moves by league, team, player and more.",
      },
    },
    { path: "/teams/:name", component: () => import("./views/TeamView.vue"), props: true },
    { path: "/players/:name", component: () => import("./views/PlayerView.vue"), props: true },
    {
      path: "/privacy",
      component: () => import("./views/PrivacyView.vue"),
      meta: { title: "Privacy" },
    },
    {
      path: "/credits",
      component: () => import("./views/CreditsView.vue"),
      meta: { title: "Data and credits" },
    },
    {
      path: "/rumors",
      component: () => import("./views/RumorsView.vue"),
      meta: {
        title: "Rumor tracker",
        description: "Track open, confirmed and denied League of Legends esports transfer rumors.",
      },
    },
    {
      path: "/rumors/new",
      component: () => import("./views/SubmitRumorView.vue"),
      meta: { title: "Send a rumor" },
    },
  ],
  scrollBehavior: (to, from, saved) => saved ?? (to.path === from.path ? undefined : { top: 0 }),
});

const description = document.querySelector<HTMLMetaElement>('meta[name="description"]')!;
const defaultDescription = description.content;
const canonical = document.createElement("link");

canonical.rel = "canonical";

router.afterEach((to) => {
  const name = to.params.name;
  const page = (Array.isArray(name) ? name[0] : name) ?? to.meta.title;
  const path = to.path.toLowerCase();

  document.title = page ? `${page} · Transfer Board` : "Transfer Board";
  description.content = to.meta.description ?? defaultDescription;

  if (path === "/" || path === "/rumors" || (path === "/feed" && !Object.keys(to.query).length)) {
    canonical.href = new URL(path, "https://lol-market-changes.vercel.app").href;
    document.head.append(canonical);
  } else {
    canonical.remove();
  }
});
