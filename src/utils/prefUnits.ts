/** Parse preference display strings like `"10 s"`, `"125%"`, `"1.25"`. */

export function parseSeconds(raw: string | null | undefined, fallbackSec: number): number {
  if (raw == null || raw === "") return fallbackSec;
  const m = String(raw)
    .trim()
    .match(/^(-?\d+(?:\.\d+)?)\s*s(?:ec(?:onds?)?)?$/i);
  if (m) {
    const n = Number(m[1]);
    return Number.isFinite(n) ? n : fallbackSec;
  }
  const n = Number(String(raw).trim());
  return Number.isFinite(n) ? n : fallbackSec;
}

export function parseSecondsMs(raw: string | null | undefined, fallbackMs: number): number {
  return Math.round(parseSeconds(raw, fallbackMs / 1000) * 1000);
}

export function parsePercent(raw: string | null | undefined, fallback: number): number {
  if (raw == null || raw === "") return fallback;
  const m = String(raw)
    .trim()
    .match(/^(-?\d+(?:\.\d+)?)\s*%?$/);
  if (!m) return fallback;
  const n = Number(m[1]);
  return Number.isFinite(n) ? n : fallback;
}
