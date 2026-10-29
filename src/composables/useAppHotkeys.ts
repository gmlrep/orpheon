import { onMounted, onUnmounted } from "vue";
import { useUiStore } from "../stores/ui";
import { usePlaybackStore } from "../stores/playback";
import { useSettingsStore } from "../stores/settings";
import { useFullscreen } from "./useFullscreen";
import { HOTKEY_PROFILE } from "../config/app";

export function useAppHotkeys() {
  const ui = useUiStore();
  const playback = usePlaybackStore();
  const settings = useSettingsStore();
  const { toggleFullscreen } = useFullscreen();

  function isMediaOnly(): boolean {
    return settings.hotkeyProfile() === HOTKEY_PROFILE.media;
  }

  function onKey(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA") return;

    const mediaOnly = isMediaOnly();
    const mod = e.ctrlKey || e.metaKey || e.altKey;

    if (!mediaOnly && e.key === "[" && !mod) {
      e.preventDefault();
      ui.cycleSidebar();
    }
    if (e.code === "Space") {
      if (playback.session.active) {
        e.preventDefault();
        void playback.playPause();
      }
    }
    if (e.key === "F11" || (!mediaOnly && e.key === "f" && !mod)) {
      e.preventDefault();
      void toggleFullscreen();
    }
    if (e.key === "Escape") {
      if (ui.fullscreen) {
        e.preventDefault();
        void toggleFullscreen();
      } else if (playback.tracksOpen) {
        playback.setTracksOpen(false);
      }
    }
    if (e.key === "MediaTrackNext" || (!mediaOnly && e.key === "n" && !mod)) {
      if (playback.session.active) void playback.nextTrack();
    }
    if (e.key === "MediaTrackPrevious" || (!mediaOnly && e.key === "p" && !mod)) {
      if (playback.session.active) void playback.previousTrack();
    }

    if (!mediaOnly && playback.session.active && !e.ctrlKey && !e.metaKey) {
      if (e.key === "ArrowLeft") {
        e.preventDefault();
        const which = e.altKey ? "long" : e.shiftKey ? "medium" : "short";
        void playback.seekRelative(-settings.skipMs(which));
      }
      if (e.key === "ArrowRight") {
        e.preventDefault();
        const which = e.altKey ? "long" : e.shiftKey ? "medium" : "short";
        void playback.seekRelative(settings.skipMs(which));
      }
      if (e.key === "ArrowUp" && !e.altKey) {
        e.preventDefault();
        void playback.nudgeVolume(settings.volumeStep());
      }
      if (e.key === "ArrowDown" && !e.altKey) {
        e.preventDefault();
        void playback.nudgeVolume(-settings.volumeStep());
      }
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", onKey);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", onKey);
  });
}
