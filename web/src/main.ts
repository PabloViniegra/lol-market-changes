import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { createPinia } from "pinia";
import { createApp } from "vue";
import App from "./App.vue";
import { HttpError } from "./api/client";
import { FEED_PAGES_KEY } from "./api/feedRefresh";
import { router } from "./router";
import "./style.css";

function isClientError(error: Error | null): boolean {
  return error instanceof HttpError && error.status < 500;
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60_000,
      refetchInterval: (query) => {
        if (query.state.status !== "error") {
          return query.queryKey[0] === FEED_PAGES_KEY[0] ? 15 * 60_000 : 5 * 60_000;
        }

        return isClientError(query.state.error) ? false : 30_000;
      },
      retry: (failures, error) => !isClientError(error) && failures < 2,
    },
  },
});

createApp(App).use(createPinia()).use(router).use(VueQueryPlugin, { queryClient }).mount("#app");
