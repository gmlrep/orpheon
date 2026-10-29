import { computed, type Ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";

export function useCoverSrc(coverPath: Ref<string | null | undefined>) {
  return computed(() => {
    const path = coverPath.value;
    if (!path) return null;
    return convertFileSrc(path.replace(/\\/g, "/"));
  });
}
