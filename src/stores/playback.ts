import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { MediaItem, PlaybackTick, SessionInfo, TrackSnapshot } from "../types";
import { useUiStore } from "./ui";

export interface QueueItem {
  path: string;
  title: string;
  mediaId?: number;
  isVideo: boolean;
  artworkPath?: string | null;
}

const emptySession = (): SessionInfo => ({
  active: false,
  paused: true,
  title: "",
  subtitle: "",
  path: "",
  mediaId: null,
  isVideo: true,
  position: 0,
  timeMs: 0,
  lengthMs: 0,
  volume: 80,
});

async function tearDownVideoSurface() {
  await invoke("hide_video_surface");
  await invoke("update_stage_rect", {
    rect: { x: 0, y: 0, width: 0, height: 0, visible: false, scaleFactor: 1 },
  });
  await invoke("set_overlay_holes", { holes: [] });
}

export const usePlaybackStore = defineStore("playback", () => {
  const engineAvailable = ref(false);
  const session = ref<SessionInfo>(emptySession());
  const playing = ref(false);
  const tracks = ref<TrackSnapshot | null>(null);
  const queue = ref<QueueItem[]>([]);
  const queueIndex = ref(-1);
  const coverPath = ref<string | null>(null);
  /** While true, tick position updates do not overwrite local scrubbing UI. */
  const scrubbing = ref(false);
  const muted = ref(false);
  let volumeBeforeMute = 80;
  let unlistenTick: UnlistenFn | null = null;
  let unlistenEnded: UnlistenFn | null = null;
  let unlistenHotkey: UnlistenFn | null = null;
  let unlistenStopped: UnlistenFn | null = null;

  const showNowBar = computed(() => {
    const ui = useUiStore();
    return session.value.active && ui.route !== "player";
  });

  async function bootstrap() {
    try {
      const status = await invoke<{ available: boolean; session: SessionInfo }>("engine_status");
      engineAvailable.value = status.available;
      session.value = status.session ?? emptySession();
      muted.value = (session.value.volume ?? 0) <= 0;
      if (session.value.path) {
        void refreshCover(session.value.path, session.value.mediaId);
      }
    } catch {
      engineAvailable.value = false;
    }
    if (!unlistenTick) {
      unlistenTick = await listen<PlaybackTick>("playback://tick", (ev) => {
        const next = ev.payload.session;
        if (scrubbing.value) {
          session.value = {
            ...next,
            position: session.value.position,
            timeMs: session.value.timeMs,
          };
        } else {
          session.value = next;
        }
        playing.value = ev.payload.playing;
        if (!muted.value) {
          /* keep mute flag when volume is intentionally 0 */
        } else if (next.volume > 0) {
          muted.value = false;
        }
      });
    }
    if (!unlistenEnded) {
      unlistenEnded = await listen("playback://ended", () => {
        void nextTrack();
      });
    }
    if (!unlistenHotkey) {
      unlistenHotkey = await listen<string>("playback://hotkey", (ev) => {
        if (ev.payload === "next") void nextTrack();
        else if (ev.payload === "previous") void previousTrack();
        else if (ev.payload === "minimize") void minimizePlayer();
        else if (ev.payload === "toggleUi") {
          const ui = useUiStore();
          ui.uiHidden = !ui.uiHidden;
        }
      });
    }
    if (!unlistenStopped) {
      unlistenStopped = await listen("playback://stopped", () => {
        session.value = emptySession();
        playing.value = false;
        coverPath.value = null;
        queue.value = [];
        queueIndex.value = -1;
      });
    }
  }

  function setQueue(items: QueueItem[], startIndex = 0) {
    queue.value = items;
    queueIndex.value = startIndex;
  }

  async function refreshCover(path: string, mediaId?: number | null) {
    if (!path) {
      coverPath.value = null;
      return;
    }
    try {
      const art = await invoke<string | null>("resolve_artwork", {
        path,
        mediaId: mediaId ?? null,
      });
      coverPath.value = art;
    } catch {
      coverPath.value = null;
    }
  }

  async function ensureSurfaceForVideo(isVideo: boolean) {
    if (!isVideo) return;
    await invoke("ensure_video_surface");
  }

  async function afterPlay(item: QueueItem, expandVideo: boolean) {
    const ui = useUiStore();
    playing.value = true;
    void refreshCover(item.path, item.mediaId);
    if (item.mediaId != null) {
      try {
        await invoke("touch_recent", { mediaId: item.mediaId });
      } catch {
        /* ignore */
      }
    }
    if (session.value.isVideo && expandVideo) {
      ui.navigate("player");
    }
  }

  async function playQueueItem(index: number, { expandVideo = true } = {}) {
    const item = queue.value[index];
    if (!item) return;
    queueIndex.value = index;
    try {
      await ensureSurfaceForVideo(item.isVideo);
    } catch (e) {
      console.error("ensure_video_surface failed", e);
      throw e;
    }
    const next = await invoke<SessionInfo>("play_path", {
      path: item.path,
      title: item.title,
      mediaId: item.mediaId ?? null,
      isVideo: item.isVideo,
    });
    session.value = next;
    muted.value = next.volume <= 0;
    await afterPlay(item, expandVideo);
  }

  async function playPath(
    path: string,
    opts?: {
      title?: string;
      mediaId?: number;
      isVideo?: boolean;
      artworkPath?: string | null;
      queue?: QueueItem[];
      queueIndex?: number;
    },
  ) {
    if (opts?.queue?.length) {
      setQueue(opts.queue, opts.queueIndex ?? 0);
      await playQueueItem(queueIndex.value, { expandVideo: true });
      return;
    }

    setQueue(
      [
        {
          path,
          title: opts?.title ?? path,
          mediaId: opts?.mediaId,
          isVideo: opts?.isVideo ?? true,
          artworkPath: opts?.artworkPath,
        },
      ],
      0,
    );
    await playQueueItem(0, { expandVideo: true });
  }

  async function playMrl(mrl: string) {
    const ui = useUiStore();
    queue.value = [];
    queueIndex.value = -1;
    coverPath.value = null;
    try {
      await ensureSurfaceForVideo(true);
    } catch (e) {
      console.error("ensure_video_surface failed", e);
    }
    const next = await invoke<SessionInfo>("play_mrl", { mrl });
    session.value = next;
    playing.value = true;
    muted.value = next.volume <= 0;
    if (next.isVideo) ui.navigate("player");
  }

  async function playPause() {
    playing.value = await invoke<boolean>("play_pause");
    session.value = { ...session.value, paused: !playing.value, active: true };
  }

  async function stop() {
    await invoke("stop_playback");
    try {
      await tearDownVideoSurface();
    } catch {
      /* ignore */
    }
    session.value = emptySession();
    playing.value = false;
    coverPath.value = null;
    queue.value = [];
    queueIndex.value = -1;
  }

  async function seek(position: number) {
    await invoke("seek", { position });
    session.value = { ...session.value, position };
  }

  function beginScrub() {
    scrubbing.value = true;
  }

  function endScrub() {
    scrubbing.value = false;
  }

  async function seekRelative(deltaMs: number) {
    await invoke("seek_relative", { deltaMs });
  }

  async function nextTrack() {
    if (!queue.value.length) return;
    const next = queueIndex.value + 1;
    if (next >= queue.value.length) return;
    const stayMini = useUiStore().route !== "player";
    await playQueueItem(next, { expandVideo: !stayMini && !!queue.value[next]?.isVideo });
    if (stayMini && queue.value[next]?.isVideo) {
      try {
        await tearDownVideoSurface();
      } catch {
        /* ignore */
      }
    }
  }

  async function previousTrack() {
    if (!queue.value.length) return;
    if (session.value.timeMs > 3000) {
      await seek(0);
      return;
    }
    const prev = queueIndex.value - 1;
    if (prev < 0) {
      await seek(0);
      return;
    }
    const stayMini = useUiStore().route !== "player";
    await playQueueItem(prev, { expandVideo: !stayMini && !!queue.value[prev]?.isVideo });
    if (stayMini && queue.value[prev]?.isVideo) {
      try {
        await tearDownVideoSurface();
      } catch {
        /* ignore */
      }
    }
  }

  async function setVolume(volume: number) {
    muted.value = volume <= 0;
    await invoke("set_volume", { volume });
    session.value = { ...session.value, volume };
  }

  async function toggleMute() {
    if (!muted.value) {
      volumeBeforeMute = session.value.volume || 80;
      muted.value = true;
      await invoke("set_volume", { volume: 0 });
      session.value = { ...session.value, volume: 0 };
    } else {
      muted.value = false;
      const v = volumeBeforeMute || 80;
      await invoke("set_volume", { volume: v });
      session.value = { ...session.value, volume: v };
    }
  }

  async function minimizePlayer() {
    const ui = useUiStore();
    if (ui.fullscreen) {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        await getCurrentWindow().setFullscreen(false);
      } catch {
        /* ignore */
      }
      ui.setFullscreen(false);
    }
    try {
      await tearDownVideoSurface();
    } catch {
      /* ignore */
    }
    const dest = session.value.isVideo ? "videos" : "music";
    if (ui.route === "player") {
      ui.navigate(dest);
    }
    void refreshCover(session.value.path, session.value.mediaId);
  }

  async function openPlayer() {
    const ui = useUiStore();
    if (!session.value.active) return;
    if (session.value.isVideo) {
      await ensureSurfaceForVideo(true);
    }
    ui.navigate("player");
  }

  async function refreshTracks() {
    tracks.value = await invoke<TrackSnapshot>("track_snapshot");
  }

  async function setTrack(kind: string, id: number) {
    await invoke("set_track", { kind, id });
    await refreshTracks();
  }

  async function applyEqualizer(enabled: boolean, preamp: number, bands: number[]) {
    await invoke("apply_equalizer", { enabled, preamp, bands });
  }

  function mediaToQueue(items: MediaItem[], isVideo: boolean): QueueItem[] {
    return items.map((m) => ({
      path: m.path,
      title: m.title,
      mediaId: m.id,
      isVideo: isVideo || m.media_type !== "audio",
      artworkPath: m.artwork_path,
    }));
  }

  return {
    engineAvailable,
    session,
    playing,
    tracks,
    queue,
    queueIndex,
    coverPath,
    scrubbing,
    showNowBar,
    bootstrap,
    setQueue,
    playPath,
    playMrl,
    playPause,
    stop,
    seek,
    beginScrub,
    endScrub,
    seekRelative,
    nextTrack,
    previousTrack,
    setVolume,
    muted,
    toggleMute,
    minimizePlayer,
    openPlayer,
    refreshTracks,
    setTrack,
    applyEqualizer,
    mediaToQueue,
    refreshCover,
    tearDownVideoSurface,
  };
});
