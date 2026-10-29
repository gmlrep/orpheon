import { computed, type Ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { sanitizeArtworkUrl } from "../utils/mediaUrl";

/**
 * Resolve cover art for `<img src>`.
 * - Remote stream art: `http(s)` only
 * - Local cache paths from the artwork pipeline: `convertFileSrc`
 */
export function resolveMediaSrc(path: string | null | undefined): string | null {
  if (!path) return null;
  const trimmed = path.trim();
  if (!trimmed) return null;

  const remote = sanitizeArtworkUrl(trimmed);
  if (remote) return remote;

  // Reject URI schemes from untrusted metadata, but keep Windows drive paths (`C:\…`).
  const windowsPath = /^[a-zA-Z]:[\\/]/.test(trimmed);
  const uriScheme = /^[a-z][a-z0-9+.-]*:/i.test(trimmed);
  if (uriScheme && !windowsPath) return null;

  return convertFileSrc(trimmed.replace(/\\/g, "/"));
}

export function useCoverSrc(coverPath: Ref<string | null | undefined>) {
  return computed(() => resolveMediaSrc(coverPath.value));
}
