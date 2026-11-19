import { onScopeDispose, ref } from "vue";

export function useMediaQuery(query: string) {
  const list = window.matchMedia(query);
  const matches = ref(list.matches);
  const update = (event: MediaQueryListEvent) => (matches.value = event.matches);

  list.addEventListener("change", update);
  onScopeDispose(() => list.removeEventListener("change", update));

  return matches;
}
