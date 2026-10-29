import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useSettingsStore } from "./settings";
import { ACCENT_SWATCHES } from "../settings/accents";
import { mockMatchMedia } from "../test/helpers";

const invokeMock = vi.mocked(invoke);
const openUrlMock = vi.mocked(openUrl);

describe("settings store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
    openUrlMock.mockReset();
    document.documentElement.removeAttribute("data-theme");
    document.documentElement.removeAttribute("data-reduce-motion");
    document.documentElement.style.removeProperty("--accent");
    mockMatchMedia(() => false);
  });

  it("load merges defaults with prefs and marks loaded", async () => {
    invokeMock.mockResolvedValueOnce({
      theme: "light",
      volume: 40,
      hw_accel: false,
      resume_playback: false,
      language: "ru",
      vlc_options: ["--foo"],
      prefs: { accent_index: 2 },
    });

    const store = useSettingsStore();
    await store.load();

    expect(store.loaded).toBe(true);
    expect(store.settings.theme).toBe("light");
    expect(store.settings.volume).toBe(40);
    expect(store.prefNumber("accent_index")).toBe(2);
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(document.documentElement.style.getPropertyValue("--accent")).toBe(
      ACCENT_SWATCHES[2],
    );
  });

  it("load falls back to defaults when invoke fails", async () => {
    invokeMock.mockRejectedValueOnce(new Error("offline"));

    const store = useSettingsStore();
    await store.load();

    expect(store.loaded).toBe(false);
    expect(store.settings.theme).toBe("dark");
    expect(store.settings.volume).toBe(80);
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("setPref accent_index updates CSS accent", async () => {
    invokeMock.mockResolvedValue(undefined);
    const store = useSettingsStore();

    await store.setPref("accent_index", 1);

    expect(invokeMock).toHaveBeenCalledWith("save_settings", expect.any(Object));
    expect(store.prefNumber("accent_index")).toBe(1);
    expect(document.documentElement.style.getPropertyValue("--accent")).toBe(
      ACCENT_SWATCHES[1],
    );
  });

  it("setPref mirrors resume_playback and hw_accel into top-level settings", async () => {
    invokeMock.mockResolvedValue(undefined);
    const store = useSettingsStore();

    await store.setPref("resume_playback", false);
    expect(store.settings.resume_playback).toBe(false);
    expect(store.prefBool("resume_playback")).toBe(false);

    await store.setPref("hw_accel", false);
    expect(store.settings.hw_accel).toBe(false);
    expect(store.prefBool("hw_accel")).toBe(false);
  });

  it("cyclePref walks options and wraps", async () => {
    invokeMock.mockResolvedValue(undefined);
    const store = useSettingsStore();
    const options = ["a", "b", "c"];

    await store.cyclePref("hotkey_profile", options);
    expect(store.prefString("hotkey_profile")).toBe("b");

    await store.cyclePref("hotkey_profile", options);
    expect(store.prefString("hotkey_profile")).toBe("c");

    await store.cyclePref("hotkey_profile", options);
    expect(store.prefString("hotkey_profile")).toBe("a");
  });

  it("runAction reset_picture clears picture prefs", async () => {
    invokeMock.mockResolvedValue(undefined);
    const store = useSettingsStore();

    await store.runAction("reset_picture");

    expect(store.prefString("hue")).toBe("0");
    expect(store.prefString("brightness")).toBe("1.0");
    expect(store.prefString("contrast")).toBe("1.0");
    expect(store.prefString("saturation")).toBe("1.0");
    expect(store.prefString("gamma")).toBe("1.0");
  });

  it("runAction find_extensions opens Videolan URL", async () => {
    openUrlMock.mockResolvedValue(undefined);
    const store = useSettingsStore();

    await store.runAction("find_extensions");

    expect(openUrlMock).toHaveBeenCalledWith(
      "https://www.videolan.org/vlc/download-windows.html",
    );
  });

  it("runAction clear_history invokes backend", async () => {
    invokeMock.mockResolvedValue(undefined);
    const store = useSettingsStore();

    await store.runAction("clear_history");

    expect(invokeMock).toHaveBeenCalledWith("clear_history");
  });

  it("setPref continue_mode Never clears resume_playback", async () => {
    invokeMock.mockResolvedValue(undefined);
    const store = useSettingsStore();

    await store.setPref("continue_mode", "Never");
    expect(store.settings.resume_playback).toBe(false);
    expect(store.prefString("continue_mode")).toBe("Never");
  });

  it("setTheme updates dataset and invokes backend", async () => {
    invokeMock.mockResolvedValue(undefined);
    const store = useSettingsStore();

    await store.setTheme("light");

    expect(invokeMock).toHaveBeenCalledWith("set_theme", { theme: "light" });
    expect(store.settings.theme).toBe("light");
    expect(document.documentElement.dataset.theme).toBe("light");
  });
});
