import { onScopeDispose, shallowRef } from "vue";

export function useNow(intervalMs = 60_000) {
  const now = shallowRef(new Date());
  const timer = setInterval(() => (now.value = new Date()), intervalMs);

  onScopeDispose(() => clearInterval(timer));

  return now;
}
