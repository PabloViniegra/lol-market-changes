import { createRouter, createWebHistory } from "vue-router";
import DashboardView from "./views/DashboardView.vue";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", component: DashboardView, meta: { title: "Dashboard" } },
    {
      path: "/feed",
      component: () => import("./views/FeedView.vue"),
      meta: { title: "Market feed" },
    },
    { path: "/teams/:name", component: () => import("./views/TeamView.vue"), props: true },
    { path: "/players/:name", component: () => import("./views/PlayerView.vue"), props: true },
    {
      path: "/rumors",
      component: () => import("./views/RumorsView.vue"),
      meta: { title: "Rumor tracker" },
    },
  ],
  scrollBehavior: (to, from, saved) => saved ?? (to.path === from.path ? undefined : { top: 0 }),
});

router.afterEach((to) => {
  const name = to.params.name;
  const page = (Array.isArray(name) ? name[0] : name) ?? to.meta.title;

  document.title = page ? `${page} · Transfer Board` : "Transfer Board";
});
