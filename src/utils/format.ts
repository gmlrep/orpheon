export function formatDuration(ms: number | null | undefined): string {
  if (ms == null || ms < 0 || !Number.isFinite(ms)) return "—";
  const total = Math.floor(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return `${m}:${String(s).padStart(2, "0")}`;
}

export function initials(title: string): string {
  return title
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0])
    .join("")
    .toUpperCase();
}

const PALETTES: [string, string, string][] = [
  ["#2a1c0e", "#d37918", "#1a1008"],
  ["#142028", "#6a9bb8", "#0c1218"],
  ["#1e1410", "#c4a574", "#120c08"],
  ["#1a1020", "#a878c8", "#0e0814"],
  ["#101818", "#5a9a78", "#080e0c"],
  ["#181410", "#d4b060", "#0e0a06"],
];

export function toneFor(key: string | number): [string, string, string] {
  const n = typeof key === "number" ? key : hash(key);
  return PALETTES[Math.abs(n) % PALETTES.length];
}

function hash(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
  return h;
}
