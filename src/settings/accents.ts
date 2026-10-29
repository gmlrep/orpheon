export const ACCENT_SWATCHES = [
  "#d37918",
  "#e15032",
  "#c49e23",
  "#19a234",
  "#18aad3",
  "#185dd3",
  "#8e42ff",
  "#f052b3",
  "#afafaf",
] as const;

export function accentColor(index: number): string {
  return (
    ACCENT_SWATCHES[Math.max(0, Math.min(ACCENT_SWATCHES.length - 1, index))] ?? ACCENT_SWATCHES[0]
  );
}
