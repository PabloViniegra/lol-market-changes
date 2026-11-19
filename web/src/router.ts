import { START_LOCATION, createRouter, createWebHistory } from "vue-router";
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
    {
      path: "/compare",
      component: () => import("./views/CompareView.vue"),
      meta: {
        title: "Compare rosters",
        description: "Compare the current rosters of two League of Legends esports teams.",
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
  scrollBehavior: (to, from, saved) =>
    saved ?? (to.hash ? { el: to.hash, top: 24 } : to.path === from.path ? undefined : { top: 0 }),
});

const root = document.documentElement;
let finishScreenTurn: (() => void) | undefined;
let currentScreenTurn: ViewTransition | undefined;

function setScreenHinge(snapshot: "old" | "new") {
  const top = document.getElementById("main-content")?.getBoundingClientRect().top ?? 0;

  root.style.setProperty(`--screen-hinge-${snapshot}`, `${innerHeight / 2 - top}px`);
}

router.beforeResolve(async (to, from) => {
  if (!document.startViewTransition || from === START_LOCATION || to.path === from.path) return;

  setScreenHinge("old");
  // The names are only set during a turn: a view-transition-name always creates a stacking context, which would put the footer above menus opened in main.
  root.classList.add("screen-turn");

  await new Promise<void>((captured) => {
    const transition = document.startViewTransition(
      () =>
        new Promise<void>((done) => {
          finishScreenTurn = done;
          captured();
        }),
    );

    currentScreenTurn = transition;
    void transition.finished.finally(() => {
      if (currentScreenTurn !== transition) return;

      currentScreenTurn = undefined;
      root.classList.remove("screen-turn");
    });
  });
});

router.afterEach(async () => {
  const finish = finishScreenTurn;

  finishScreenTurn = undefined;

  if (!finish) return;

  // The router applies scrollBehavior a few microtasks after nextTick; a macrotask lets the new hinge see the final scroll.
  await new Promise((resolve) => setTimeout(resolve));
  setScreenHinge("new");
  finish();
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
