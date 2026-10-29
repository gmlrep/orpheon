import { nextTick, onBeforeUnmount, onMounted, toValue, watch, type MaybeRefOrGetter } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { storeToRefs } from "pinia";
import { useUiStore } from "../stores/ui";
import { usePlaybackStore } from "../stores/playback";
import { SIDEBAR_TRANSITION_MS } from "../config/app";

/** Extra frames after `--t-nav` so the HWND settles on the final layout. */
const LAYOUT_FOLLOW_TAIL_MS = 48;

export function useVideoStage(options: {
  stageEl: MaybeRefOrGetter<HTMLElement | null | undefined>;
  controlsEl: MaybeRefOrGetter<HTMLElement | null | undefined>;
  isMusic: MaybeRefOrGetter<boolean>;
  chromeVisible: MaybeRefOrGetter<boolean>;
}) {
  const ui = useUiStore();
  const playback = usePlaybackStore();
  const { fullscreen, uiHidden, sidebarMode } = storeToRefs(ui);
  const { tracksOpen } = storeToRefs(playback);

  let raf = 0;
  let followRaf = 0;
  let followUntil = 0;
  let flushing = false;
  let dirty = false;
  let unlistenResize: (() => void) | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let observedEl: HTMLElement | null = null;

  async function syncStageOnce() {
    const stage = toValue(options.stageEl);
    if (!stage) return;

    const box = (fullscreen.value ? stage.closest(".player-stage") : null) as HTMLElement | null;
    const target = box ?? stage;
    const rect = target.getBoundingClientRect();
    const scale = window.devicePixelRatio || 1;
    const width = Math.max(1, Math.round(rect.width));
    const height = Math.max(1, Math.round(rect.height));

    await invoke("update_stage_rect", {
      rect: {
        x: Math.round(rect.left),
        y: Math.round(rect.top),
        width,
        height,
        visible: !toValue(options.isMusic),
        scaleFactor: scale,
      },
    });

    const holes: number[][] = [];
    const controls = toValue(options.controlsEl);
    const chrome = toValue(options.chromeVisible);
    if (controls && chrome) {
      const c = controls.getBoundingClientRect();
      holes.push([
        Math.round((c.left - rect.left) * scale),
        Math.round((c.top - rect.top) * scale),
        Math.round(c.width * scale),
        Math.round(c.height * scale),
        0,
      ]);
    }
    const sheet = document.querySelector(".tracks-sheet.is-open") as HTMLElement | null;
    if (sheet && chrome) {
      const s = sheet.getBoundingClientRect();
      holes.push([
        Math.round((s.left - rect.left) * scale),
        Math.round((s.top - rect.top) * scale),
        Math.round(s.width * scale),
        Math.round(s.height * scale),
        8,
      ]);
    }
    await invoke("set_overlay_holes", { holes });
  }

  async function flushSync() {
    if (flushing) return;
    flushing = true;
    try {
      // Coalesce: always apply the latest layout; drop intermediate IPC if more
      // frames arrived while we were awaiting (avoids HWND jumps from out-of-order).
      while (dirty) {
        dirty = false;
        await syncStageOnce();
      }
    } finally {
      flushing = false;
      if (dirty) void flushSync();
    }
  }

  function scheduleSync() {
    dirty = true;
    cancelAnimationFrame(raf);
    raf = requestAnimationFrame(() => {
      void flushSync();
    });
  }

  function followLayout(durationMs: number) {
    followUntil = performance.now() + durationMs;
    cancelAnimationFrame(followRaf);

    const tick = () => {
      scheduleSync();
      if (performance.now() < followUntil) {
        followRaf = requestAnimationFrame(tick);
      } else {
        // One last sync after the CSS transition settles.
        scheduleSync();
      }
    };
    followRaf = requestAnimationFrame(tick);
  }

  function bindResizeObserver(el: HTMLElement | null | undefined) {
    if (observedEl === el) return;
    resizeObserver?.disconnect();
    observedEl = el ?? null;
    if (!el) return;
    resizeObserver = new ResizeObserver(() => scheduleSync());
    resizeObserver.observe(el);
    const stageRoot = el.closest(".player-stage");
    if (stageRoot instanceof HTMLElement && stageRoot !== el) {
      resizeObserver.observe(stageRoot);
    }
  }

  onMounted(async () => {
    await nextTick();
    bindResizeObserver(toValue(options.stageEl));
    scheduleSync();
    window.addEventListener("resize", scheduleSync);
    const win = getCurrentWindow();
    unlistenResize = await win.onResized(() => scheduleSync());
  });

  onBeforeUnmount(() => {
    window.removeEventListener("resize", scheduleSync);
    cancelAnimationFrame(raf);
    cancelAnimationFrame(followRaf);
    resizeObserver?.disconnect();
    resizeObserver = null;
    observedEl = null;
    unlistenResize?.();
  });

  watch(
    () => toValue(options.stageEl),
    (el) => {
      bindResizeObserver(el);
      scheduleSync();
    },
  );

  watch(
    () => sidebarMode.value,
    () => {
      // Native HWND sits above the WebView — it must track the canvas width for
      // every frame of the sidebar grid transition, or the sidebar slides under video.
      followLayout(SIDEBAR_TRANSITION_MS + LAYOUT_FOLLOW_TAIL_MS);
    },
  );

  watch(
    () => [tracksOpen.value, fullscreen.value, toValue(options.chromeVisible), uiHidden.value],
    () => scheduleSync(),
  );

  return { syncStage: syncStageOnce, scheduleSync };
}
