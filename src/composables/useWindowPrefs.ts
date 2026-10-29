import { watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { storeToRefs } from "pinia";
import { useSettingsStore } from "../stores/settings";
import { usePlaybackStore } from "../stores/playback";
import { APP_NAME } from "../config/app";

/** Apply window-level prefs (always-on-top, title bar text, decorations). */
export function useWindowPrefs() {
  const settingsStore = useSettingsStore();
  const playback = usePlaybackStore();
  const { settings } = storeToRefs(settingsStore);
  const { session, playing } = storeToRefs(playback);

  async function applyKeepAbove() {
    try {
      const on = settingsStore.prefBool("keep_above", false) && session.value.active;
      await getCurrentWindow().setAlwaysOnTop(on);
    } catch {
      /* ignore */
    }
  }

  async function applyTitle() {
    try {
      const showName = settingsStore.prefBool("show_filename_title", true);
      const title =
        showName && session.value.active && session.value.title
          ? `${session.value.title} — ${APP_NAME}`
          : APP_NAME;
      await getCurrentWindow().setTitle(title);
    } catch {
      /* ignore */
    }
  }

  async function applyDecorations() {
    try {
      const on = settingsStore.prefBool("show_decorations", false);
      await getCurrentWindow().setDecorations(on);
    } catch {
      /* ignore */
    }
  }

  watch(
    [settings, session, playing],
    () => {
      void applyKeepAbove();
      void applyTitle();
      void applyDecorations();
    },
    { immediate: true, deep: true },
  );
}
