import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../types";

const defaults = (): AppSettings => ({
  theme: "dark",
  volume: 80,
  hw_accel: true,
  resume_playback: true,
  language: "system",
  vlc_options: [],
  prefs: {},
});

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>(defaults());
  const loaded = ref(false);

  async function load() {
    try {
      settings.value = await invoke<AppSettings>("get_settings");
      loaded.value = true;
      applyTheme(settings.value.theme);
    } catch {
      settings.value = defaults();
    }
  }

  async function save(next?: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...next };
    await invoke("save_settings", { settings: settings.value });
    applyTheme(settings.value.theme);
  }

  async function setTheme(theme: AppSettings["theme"]) {
    await invoke("set_theme", { theme });
    settings.value = { ...settings.value, theme };
    applyTheme(theme);
  }

  async function setLanguage(language: string) {
    await invoke("set_language", { language });
    settings.value = { ...settings.value, language };
  }

  function applyTheme(theme: AppSettings["theme"]) {
    const dark =
      theme === "dark" ||
      (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.dataset.theme = dark ? "dark" : "light";
  }

  return { settings, loaded, load, save, setTheme, setLanguage };
});
