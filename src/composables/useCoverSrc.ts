import { computed, type Ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";

/** Resolve a local path, `file://`, or remote `http(s)` URL for `<img src>`. */
export function resolveMediaSrc(path: string | null | undefined): string | null {
  if (!path) return null;
  const trimmed = path.trim();
  if (!trimmed) return null;
  if (/^https?:\/\//i.test(trimmed) || trimmed.startsWith("data:")) return trimmed;
  if (trimmed.startsWith("file://")) {
    try {
      const url = new URL(trimmed);
      return convertFileSrc(decodeURIComponent(url.pathname).replace(/^\/([A-Za-z]:)/, "$1"));
    } catch {
      return convertFileSrc(trimmed.replace(/^file:\/\//i, ""));
    }
  }
  return convertFileSrc(trimmed.replace(/\\/g, "/"));
}

export function useCoverSrc(coverPath: Ref<string | null | undefined>) {
  return computed(() => resolveMediaSrc(coverPath.value));
}
