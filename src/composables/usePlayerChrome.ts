import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  shallowRef,
  toValue,
  watch,
  type MaybeRefOrGetter,
} from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { storeToRefs } from "pinia";
import { useUiStore } from "../stores/ui";
import { useSettingsStore } from "../stores/settings";
import { useFullscreen } from "./useFullscreen";
import { PLAYER_CHROME } from "../config/app";

export function usePlayerChrome(options: {
  scheduleSync: () => void;
  tracksOpen: MaybeRefOrGetter<boolean>;
}) {
  const { scheduleSync } = options;
  const ui = useUiStore();
  const settings = useSettingsStore();
  const { fullscreen, uiHidden } = storeToRefs(ui);
  const { toggleFullscreen: toggleFs, syncFromOs } = useFullscreen();

  const controlsReveal = shallowRef(true);
  let hideTimer = 0;
  let fsPoll = 0;
  let cursorPoll = 0;

  const chromeVisible = computed(() => {
    if (uiHidden.value) return false;
    if (!fullscreen.value) return true;
    if (!settings.showFsControls()) return false;
    return controlsReveal.value;
  });

  function revealControls() {
    controlsReveal.value = true;
    if (uiHidden.value) ui.setUiHidden(false);
    window.clearTimeout(hideTimer);
    if (
      settings.showFsControls() &&
      settings.shouldAutohide(fullscreen.value) &&
      !toValue(options.tracksOpen)
    ) {
      const ms = settings.autohideMs();
      if (Number.isFinite(ms)) {
        hideTimer = window.setTimeout(() => {
          controlsReveal.value = false;
          scheduleSync();
        }, ms);
      }
    }
    scheduleSync();
  }

  function onStageMove() {
    if (fullscreen.value) revealControls();
  }

  async function toggleFullscreen() {
    const next = await toggleFs();
    controlsReveal.value = true;
    if (next) revealControls();
    await nextTick();
    scheduleSync();
  }

  onMounted(async () => {
    await syncFromOs();

    const win = getCurrentWindow();
    fsPoll = window.setInterval(async () => {
      try {
        const fs = await win.isFullscreen();
        if (fs !== ui.fullscreen) {
          ui.setFullscreen(fs);
          controlsReveal.value = true;
          if (fs) revealControls();
          scheduleSync();
        }
      } catch {
        /* ignore */
      }
    }, PLAYER_CHROME.fullscreenPollMs);

    cursorPoll = window.setInterval(async () => {
      if (!fullscreen.value || controlsReveal.value || toValue(options.tracksOpen)) return;
      try {
        const near = await invoke<boolean>("cursor_in_stage_bottom", {
          stripCssPx: PLAYER_CHROME.bottomStripCssPx,
        });
        if (near) revealControls();
      } catch {
        /* ignore */
      }
    }, PLAYER_CHROME.cursorPollMs);
  });

  onBeforeUnmount(() => {
    window.clearTimeout(hideTimer);
    window.clearInterval(fsPoll);
    window.clearInterval(cursorPoll);
  });

  watch(
    () => toValue(options.tracksOpen),
    (open) => {
      if (open) revealControls();
    },
  );

  return {
    controlsReveal,
    chromeVisible,
    revealControls,
    onStageMove,
    toggleFullscreen,
  };
}
