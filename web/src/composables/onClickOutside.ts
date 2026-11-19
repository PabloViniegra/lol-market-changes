import { onBeforeUnmount, onMounted, type Ref } from "vue";

export function onClickOutside(target: Ref<HTMLElement | null>, handler: () => void) {
  const listener = (event: PointerEvent) => {
    if (target.value && event.target instanceof Node && !target.value.contains(event.target))
      handler();
  };

  onMounted(() => document.addEventListener("pointerdown", listener));
  onBeforeUnmount(() => document.removeEventListener("pointerdown", listener));
}
