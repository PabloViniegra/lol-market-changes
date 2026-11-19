import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";

export const GO_TO = {
  d: { path: "/", label: "Dashboard" },
  f: { path: "/feed", label: "Feed" },
  r: { path: "/rumors", label: "Rumors" },
  c: { path: "/compare", label: "Compare" },
} as const;

const SEQUENCE_MS = 1000;

function isTyping(target: EventTarget | null) {
  return (
    target instanceof HTMLElement && target.closest("input, textarea, select, [contenteditable]")
  );
}

function focusRow(step: 1 | -1) {
  const rows = [...document.querySelectorAll<HTMLElement>("#main-content .row-link")];
  const current = rows.findIndex((row) => row === document.activeElement);
  const fallback = step > 0 ? 0 : rows.length - 1;
  const row = rows[current < 0 ? fallback : Math.min(rows.length - 1, Math.max(0, current + step))];

  if (!row) return;

  row.focus();
  row.scrollIntoView({ block: "nearest" });
}

export function useShortcuts(openHelp: () => void) {
  const router = useRouter();
  let pendingGo = 0;

  function onKeydown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey || event.altKey || event.defaultPrevented) return;

    if (isTyping(event.target) || document.querySelector("dialog[open]")) return;

    const destination = Object.entries(GO_TO).find(([key]) => key === event.key)?.[1];
    const inSequence = pendingGo > 0 && event.timeStamp - pendingGo < SEQUENCE_MS;

    pendingGo = 0;

    if (inSequence && destination) {
      event.preventDefault();
      void router.push(destination.path);
    } else if (event.key === "g") {
      pendingGo = event.timeStamp;
    } else if (event.key === "j" || event.key === "k") {
      event.preventDefault();
      focusRow(event.key === "j" ? 1 : -1);
    } else if (event.key === "?") {
      event.preventDefault();
      openHelp();
    }
  }

  onMounted(() => window.addEventListener("keydown", onKeydown));

  onUnmounted(() => window.removeEventListener("keydown", onKeydown));
}
