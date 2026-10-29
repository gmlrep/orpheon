/** Allow only remote http(s) artwork URLs from stream metadata / bookmarks. */
export function sanitizeArtworkUrl(raw: string | null | undefined): string | null {
  if (raw == null) return null;
  const url = String(raw).trim();
  if (!url) return null;
  if (/^https?:\/\//i.test(url)) return url;
  return null;
}
