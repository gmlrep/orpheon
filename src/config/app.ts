/** Shared app constants — single source for magic numbers / URLs used across UI. */

export const APP_NAME = "Orpheon";

export const VLC_DOWNLOAD_WINDOWS_URL = "https://www.videolan.org/vlc/download-windows.html";

/** Softvol: 100 = 0 dB, engine allows up to 200. */
export const SOFTVOL_UNITY = 100;
export const SOFTVOL_ENGINE_MAX = 200;
export const DEFAULT_VOLUME = 80;
export const DEFAULT_MAX_VOLUME = 125;

export const DEFAULT_THEME = "dark" as const;
export const DEFAULT_LANGUAGE = "system";

export const ART_MAX_CONCURRENT = 2;
export const RECENT_LIST_LIMIT = 100;
export const LISTEN_AGAIN_LIMIT = 8;

/** Narrow layout breakpoint (must match `@media` in shell.css). */
export const NARROW_LAYOUT_MQ = "(max-width: 900px)";

export const SEARCH_DEBOUNCE_MS = 200;
export const EQ_APPLY_DEBOUNCE_MS = 80;

/** Must match `--t-nav` in tokens.css — used to keep HWND video in sync while sidebar animates. */
export const SIDEBAR_TRANSITION_MS = 208;

export const PLAYER_CHROME = {
  /** Fallback when pref `autohide_after` is missing / unparsable. */
  autohideMs: 2200,
  fullscreenPollMs: 500,
  cursorPollMs: 120,
  bottomStripCssPx: 72,
} as const;

export const EQ_BAND_KEYS = [
  "60hz",
  "170hz",
  "310hz",
  "600hz",
  "1khz",
  "3khz",
  "6khz",
  "12khz",
  "14khz",
  "16khz",
] as const;

export const EQ_BAND_COUNT = EQ_BAND_KEYS.length;
export const EQ_DEFAULT_GAIN = 0.5;

export const HOTKEY_PROFILE = {
  vlc: "VLC defaults",
  media: "Media keys",
  custom: "Custom",
} as const;

export const DEFAULT_SKIP = {
  veryShort: "3 s",
  short: "10 s",
  medium: "60 s",
  long: "300 s",
} as const;

export const DEFAULT_AUTOHIDE_AFTER = "5 s";
export const DEFAULT_MAX_VOLUME_PREF = "125%";
export const DEFAULT_VOLUME_STEP_PREF = "5%";
export const DEFAULT_STARTUP_VOLUME_PCT = "80%";
export const STARTUP_VOLUME_REMEMBER = "Remember last volume";
