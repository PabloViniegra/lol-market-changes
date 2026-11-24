import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { createPinia } from "pinia";
import { createApp } from "vue";
import App from "./App.vue";
import { isClientError } from "./api/client";
import { router } from "./router";
import "./style.css";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Data only changes when the ingestor runs; useRefreshOnSync invalidates it when meta reports a new run.
      staleTime: Infinity,
      gcTime: 30 * 60_000,
      refetchInterval: (query) =>
        query.state.status === "error" && !isClientError(query.state.error) ? 30_000 : false,
      retry: (failures, error) => !isClientError(error) && failures < 2,
    },
  },
});

createApp(App).use(createPinia()).use(router).use(VueQueryPlugin, { queryClient }).mount("#app");
