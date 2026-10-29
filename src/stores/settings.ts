import { defineStore } from "pinia";
import { shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { AppSettings } from "../types";
import { accentColor } from "../settings/accents";
import {
  DEFAULT_AUTOHIDE_AFTER,
  DEFAULT_LANGUAGE,
  DEFAULT_MAX_VOLUME,
  DEFAULT_MAX_VOLUME_PREF,
  DEFAULT_SKIP,
  DEFAULT_THEME,
  DEFAULT_VOLUME,
  DEFAULT_VOLUME_STEP_PREF,
  EQ_BAND_COUNT,
  EQ_DEFAULT_GAIN,
  HOTKEY_PROFILE,
  PLAYER_CHROME,
  SOFTVOL_ENGINE_MAX,
  VLC_DOWNLOAD_WINDOWS_URL,
} from "../config/app";
import { parsePercent, parseSecondsMs } from "../utils/prefUnits";
import { setLocale } from "../i18n";

const SKIP_PREF: Record<
  "veryShort" | "short" | "medium" | "long",
  { key: string; fallback: string }
> = {
  veryShort: { key: "skip_very_short", fallback: DEFAULT_SKIP.veryShort },
  short: { key: "skip_short", fallback: DEFAULT_SKIP.short },
  medium: { key: "skip_medium", fallback: DEFAULT_SKIP.medium },
  long: { key: "skip_long", fallback: DEFAULT_SKIP.long },
};

const defaults = (): AppSettings => ({
  theme: DEFAULT_THEME,
  volume: DEFAULT_VOLUME,
  hw_accel: true,
  resume_playback: true,
  language: DEFAULT_LANGUAGE,
  vlc_options: [],
  prefs: {},
});

type PrefValue = string | number | boolean | null | undefined;

/** Prefs that the engine re-reads from disk on the next `save_settings` / open. */
const LIVE_ENGINE_PREFS = new Set([
  "hue",
  "brightness",
  "contrast",
  "saturation",
  "gamma",
  "aspect_ratio",
  "crop",
  "default_rate",
  "audio_delay",
  "spu_delay",
  "audio_device",
  "enable_audio",
  "enable_video",
  "max_volume",
  "equalizer_enabled",
]);

export const useSettingsStore = defineStore("settings", () => {
  const settings = shallowRef<AppSettings>(defaults());
  const loaded = shallowRef(false);
  let systemThemeMql: MediaQueryList | null = null;

  function prefs(): Record<string, PrefValue> {
    return (settings.value.prefs ?? {}) as Record<string, PrefValue>;
  }

  function prefBool(key: string, fallback = false): boolean {
    const v = prefs()[key];
    return typeof v === "boolean" ? v : fallback;
  }

  function prefString(key: string, fallback = ""): string {
    const v = prefs()[key];
    return typeof v === "string" ? v : fallback;
  }

  function prefNumber(key: string, fallback = 0): number {
    const v = prefs()[key];
    return typeof v === "number" ? v : fallback;
  }

  function skipMs(which: "veryShort" | "short" | "medium" | "long" = "short"): number {
    const { key, fallback } = SKIP_PREF[which];
    return parseSecondsMs(prefString(key, fallback), parseSecondsMs(fallback, 10_000));
  }

  function eqGains(): { preamp: number; bands: number[] } {
    const preamp = prefNumber("eq_preamp", EQ_DEFAULT_GAIN);
    const bands = Array.from({ length: EQ_BAND_COUNT }, (_, i) =>
      prefNumber(`eq_band_${i}`, EQ_DEFAULT_GAIN),
    );
    return { preamp, bands };
  }

  function maxVolume(): number {
    const pct = parsePercent(prefString("max_volume", DEFAULT_MAX_VOLUME_PREF), DEFAULT_MAX_VOLUME);
    return Math.min(SOFTVOL_ENGINE_MAX, Math.max(1, pct));
  }

  function volumeStep(): number {
    return Math.max(1, parsePercent(prefString("volume_step", DEFAULT_VOLUME_STEP_PREF), 5));
  }

  function autohideMs(): number {
    const mode = prefString("autohide_controls", "In fullscreen only");
    if (mode === "Never") return Number.POSITIVE_INFINITY;
    return parseSecondsMs(
      prefString("autohide_after", DEFAULT_AUTOHIDE_AFTER),
      PLAYER_CHROME.autohideMs,
    );
  }

  function shouldAutohide(fullscreen: boolean): boolean {
    const mode = prefString("autohide_controls", "In fullscreen only");
    if (mode === "Never") return false;
    if (mode === "Always") return true;
    return fullscreen;
  }

  function showFsControls(): boolean {
    return prefBool("show_fs_controls", true);
  }

  function hotkeyProfile(): string {
    return prefString("hotkey_profile", HOTKEY_PROFILE.vlc);
  }

  function mouseAction(which: "single" | "double"): string {
    if (which === "single") return prefString("mouse_single", "Play / Pause");
    return prefString("mouse_double", "Toggle fullscreen");
  }

  function mouseWheelMode(): string {
    return prefString("mouse_wheel", "Volume");
  }

  function mouseHScrollMode(): string {
    return prefString("mouse_hscroll", "Seek");
  }

  function mouseReverseScroll(): boolean {
    return prefBool("mouse_reverse_scroll", false);
  }

  function startFullscreen(): boolean {
    return prefBool("start_fullscreen", false);
  }

  function repeatMode(): string {
    return prefString("repeat_mode", "Off");
  }

  function stopAfterEach(): boolean {
    return prefBool("stop_after_each", false);
  }

  function onFinish(): string {
    return prefString("on_finish", "Stop");
  }

  function shouldResume(): boolean {
    if (!settings.value.resume_playback) return false;
    const mode = prefString("continue_mode", "Always");
    return mode !== "Never";
  }

  function applyAccent(index?: number) {
    const i = index ?? prefNumber("accent_index", 0);
    document.documentElement.style.setProperty("--accent", accentColor(i));
  }

  function applyReduceMotion(enabled?: boolean) {
    const on = enabled ?? prefBool("reduce_motion", false);
    document.documentElement.dataset.reduceMotion = on ? "1" : "0";
  }

  function applyTheme(theme: AppSettings["theme"] = settings.value.theme) {
    const dark =
      theme === "dark" ||
      (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.dataset.theme = dark ? "dark" : "light";
  }

  function applyAppearance() {
    applyTheme(settings.value.theme);
    applyAccent();
    applyReduceMotion();
  }

  function bindSystemThemeListener() {
    systemThemeMql?.removeEventListener("change", onSystemThemeChange);
    systemThemeMql = window.matchMedia("(prefers-color-scheme: dark)");
    systemThemeMql.addEventListener("change", onSystemThemeChange);
  }

  function onSystemThemeChange() {
    if (settings.value.theme === "system") applyTheme("system");
  }

  async function persist(next: AppSettings) {
    settings.value = next;
    await invoke("save_settings", { settings: next });
    applyAppearance();
  }

  async function load() {
    try {
      const loadedSettings = await invoke<AppSettings>("get_settings");
      settings.value = {
        ...defaults(),
        ...loadedSettings,
        prefs: { ...(loadedSettings.prefs ?? {}) },
      };
      loaded.value = true;
      applyAppearance();
      bindSystemThemeListener();
      const lang = settings.value.language;
      setLocale(lang === "system" ? navigator.language : lang);
    } catch {
      settings.value = defaults();
      applyAppearance();
    }
  }

  async function save(next?: Partial<AppSettings>) {
    const merged: AppSettings = {
      ...settings.value,
      ...next,
      prefs: {
        ...prefs(),
        ...(next?.prefs ?? {}),
      },
    };
    await persist(merged);
  }

  async function setTheme(theme: AppSettings["theme"]) {
    await invoke("set_theme", { theme });
    settings.value = { ...settings.value, theme };
    applyTheme(theme);
  }

  async function setLanguage(language: string) {
    await invoke("set_language", { language });
    settings.value = { ...settings.value, language };
    setLocale(language === "system" ? navigator.language : language);
  }

  async function setPref(key: string, value: PrefValue) {
    const patch: Partial<AppSettings> = { prefs: { [key]: value } };

    if (key === "resume_playback") {
      const on = Boolean(value);
      const curMode = prefString("continue_mode", "Always");
      patch.resume_playback = on;
      patch.prefs = {
        resume_playback: on,
        continue_mode: on ? (curMode === "Never" ? "Always" : curMode) : "Never",
      };
    } else if (key === "continue_mode") {
      const mode = String(value);
      patch.resume_playback = mode !== "Never";
      patch.prefs = { continue_mode: mode, resume_playback: mode !== "Never" };
    } else if (key === "hw_accel") {
      const on = Boolean(value);
      patch.hw_accel = on;
      patch.prefs = {
        hw_accel: on,
        hw_accel_mode: on
          ? prefString("hw_accel_mode", "Automatic") === "Disabled"
            ? "Automatic"
            : prefString("hw_accel_mode", "Automatic")
          : "Disabled",
      };
    } else if (key === "hw_accel_mode") {
      const mode = String(value);
      patch.hw_accel = mode !== "Disabled";
      patch.prefs = { hw_accel_mode: mode, hw_accel: mode !== "Disabled" };
    }

    await save(patch);
    if (key === "accent_index") applyAccent(Number(value) || 0);
    if (key === "reduce_motion") applyReduceMotion(Boolean(value));

    if (key === "equalizer_enabled") {
      try {
        const { preamp, bands } = eqGains();
        await invoke("apply_equalizer", {
          enabled: Boolean(value),
          preamp,
          bands,
        });
      } catch {
        /* ignore when idle */
      }
    }

    if (LIVE_ENGINE_PREFS.has(key) && key !== "equalizer_enabled") {
      // save_settings already re-applies session prefs on the backend
    }
  }

  async function cyclePref(key: string, options: string[]) {
    if (!options.length) return;
    const cur = prefString(key, options[0]);
    const idx = options.indexOf(cur);
    const next = options[(idx + 1) % options.length];
    await setPref(key, next);
  }

  async function runAction(action: string) {
    switch (action) {
      case "reset_picture":
        await save({
          prefs: {
            hue: "0",
            brightness: "1.0",
            contrast: "1.0",
            saturation: "1.0",
            gamma: "1.0",
          },
        });
        break;
      case "reset_shortcuts":
        await setPref("hotkey_profile", HOTKEY_PROFILE.vlc);
        break;
      case "rescan_plugins":
        try {
          await invoke("rescan_library");
        } catch {
          /* ignore */
        }
        break;
      case "find_extensions":
        try {
          await openUrl(VLC_DOWNLOAD_WINDOWS_URL);
        } catch {
          /* ignore */
        }
        break;
      case "clear_history":
        try {
          await invoke("clear_history");
        } catch {
          /* ignore */
        }
        try {
          const { useLibraryStore } = await import("./library");
          const library = useLibraryStore();
          library.recent = [];
          library.resume = [];
        } catch {
          /* ignore */
        }
        break;
      case "edit_shortcuts":
      default:
        break;
    }
  }

  return {
    settings,
    loaded,
    load,
    save,
    setTheme,
    setLanguage,
    setPref,
    cyclePref,
    runAction,
    prefBool,
    prefString,
    prefNumber,
    skipMs,
    eqGains,
    maxVolume,
    volumeStep,
    autohideMs,
    shouldAutohide,
    showFsControls,
    hotkeyProfile,
    mouseAction,
    mouseWheelMode,
    mouseHScrollMode,
    mouseReverseScroll,
    startFullscreen,
    repeatMode,
    stopAfterEach,
    onFinish,
    shouldResume,
    applyAppearance,
  };
});
