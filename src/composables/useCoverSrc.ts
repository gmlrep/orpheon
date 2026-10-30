import { computed, type Ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { DEFAULT_COVER_SRC } from "../assets/defaultCover";
import { sanitizeArtworkUrl } from "../utils/mediaUrl";

/**
 * Resolve cover art for `<img src>`.
 * - Remote stream art: `http(s)` only
 * - Local cache paths from the artwork pipeline: `convertFileSrc`
 * - Missing / rejected → `null` (use {@link resolveCoverSrc} for the bundled default)
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

/** Like {@link resolveMediaSrc}, but never empty — falls back to bundled default cover. */
export function resolveCoverSrc(path: string | null | undefined): string {
  return resolveMediaSrc(path) ?? DEFAULT_COVER_SRC;
}

export function useCoverSrc(coverPath: Ref<string | null | undefined>) {
  return computed(() => resolveCoverSrc(coverPath.value));
}
