import { ref, watch, type Ref } from "vue";

// Only ids inserted above the previous top count as new: appended pages are not new moves.
// Callers remount the board (via `key`) when the query changes, which resets `seen`.
export function useNewIds(ids: Ref<string[] | undefined>) {
  const fresh = ref(new Set<string>());
  let seen: Set<string> | null = null;

  watch(
    ids,
    (current) => {
      if (!current) return;

      if (seen) {
        const firstKnown = current.findIndex((id) => seen?.has(id));

        fresh.value = new Set(firstKnown > 0 ? current.slice(0, firstKnown) : []);
      }

      seen = new Set(current);
    },
    { immediate: true },
  );

  return fresh;
}
