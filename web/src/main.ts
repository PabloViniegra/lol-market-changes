import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { createPinia } from "pinia";
import { createApp } from "vue";
import App from "./App.vue";
import { HttpError } from "./api/client";
import { queryKeys } from "./api/keys";
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
        if (query.state.status === "error")
          return isClientError(query.state.error) ? false : 30_000;

        // Data only changes when the ingestor runs; polling meta detects that and refreshes the rest.
        return query.queryKey[0] === queryKeys.meta[0] ? 5 * 60_000 : false;
      },
      retry: (failures, error) => !isClientError(error) && failures < 2,
    },
  },
});

createApp(App).use(createPinia()).use(router).use(VueQueryPlugin, { queryClient }).mount("#app");
