import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { usePlaybackStore } from "./playback";
import { useUiStore } from "./ui";
import { mockRouterState } from "../test/routerMock";
import type { PlaybackTick, SessionInfo } from "../types";

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);

type EventHandler = (event: { payload: unknown }) => void;

function session(partial: Partial<SessionInfo> = {}): SessionInfo {
  return {
    active: true,
    paused: false,
    title: "Track",
    subtitle: "",
    path: "/media/a.mp4",
    mediaId: 1,
    isVideo: true,
    position: 0.2,
    timeMs: 1000,
    lengthMs: 5000,
    volume: 80,
    ...partial,
  };
}

describe("playback store", () => {
  const handlers = new Map<string, EventHandler>();

  beforeEach(() => {
    setActivePinia(createPinia());
    mockRouterState().reset();
    handlers.clear();
    invokeMock.mockReset();
    listenMock.mockReset();
    listenMock.mockImplementation(async (event, cb) => {
      handlers.set(event, cb as EventHandler);
      return () => handlers.delete(event);
    });
  });

  it("openTrackTab opens, toggles same tab, and switches tabs", () => {
    const store = usePlaybackStore();

    store.openTrackTab("audio");
    expect(store.tracksOpen).toBe(true);
    expect(store.trackTab).toBe("audio");
    expect(store.eqOpen).toBe(false);

    store.openTrackTab("audio");
    expect(store.tracksOpen).toBe(false);

    store.openTrackTab("subs");
    expect(store.tracksOpen).toBe(true);
    expect(store.trackTab).toBe("subs");
  });

  it("keeps scrubbing position when tick arrives", async () => {
    const store = usePlaybackStore();
    await store.bootstrap();

    store.session = session({ position: 0.1, timeMs: 500 });
    store.beginScrub();

    handlers.get("playback://tick")?.({
      payload: {
        session: session({ position: 0.9, timeMs: 4500 }),
        playing: true,
      } satisfies PlaybackTick,
    });

    expect(store.scrubbing).toBe(true);
    expect(store.session.position).toBe(0.1);
    expect(store.session.timeMs).toBe(500);
    expect(store.playing).toBe(true);

    store.endScrub();
    handlers.get("playback://tick")?.({
      payload: {
        session: session({ position: 0.9, timeMs: 4500 }),
        playing: true,
      } satisfies PlaybackTick,
    });

    expect(store.session.position).toBe(0.9);
    expect(store.session.timeMs).toBe(4500);
  });

  it("previousTrack seeks to start when past 3s", async () => {
    invokeMock.mockImplementation(async (cmd) => {
      if (cmd === "seek") return undefined;
      throw new Error(`unexpected ${String(cmd)}`);
    });

    const store = usePlaybackStore();
    store.setQueue(
      [
        { path: "/a", title: "A", isVideo: true },
        { path: "/b", title: "B", isVideo: true },
      ],
      1,
    );
    store.session = session({ timeMs: 4000, position: 0.5 });

    await store.previousTrack();

    expect(invokeMock).toHaveBeenCalledWith("seek", { position: 0 });
    expect(store.queueIndex).toBe(1);
    expect(store.session.position).toBe(0);
  });

  it("previousTrack plays previous item when early in track", async () => {
    invokeMock.mockImplementation(async (cmd, args) => {
      if (cmd === "ensure_video_surface") return undefined;
      if (cmd === "play_path") {
        return session({
          path: (args as { path: string }).path,
          title: (args as { title: string }).title,
          timeMs: 0,
          position: 0,
        });
      }
      if (cmd === "resolve_artwork") return null;
      if (cmd === "touch_recent") return undefined;
      if (cmd === "hide_video_surface") return undefined;
      if (cmd === "update_stage_rect") return undefined;
      if (cmd === "set_overlay_holes") return undefined;
      throw new Error(`unexpected ${String(cmd)}`);
    });

    mockRouterState().currentRoute.value = { name: "videos", path: "/videos" };
    const store = usePlaybackStore();
    store.setQueue(
      [
        { path: "/a", title: "A", isVideo: true, mediaId: 1 },
        { path: "/b", title: "B", isVideo: true, mediaId: 2 },
      ],
      1,
    );
    store.session = session({ timeMs: 1000, path: "/b" });

    await store.previousTrack();

    expect(store.queueIndex).toBe(0);
    expect(invokeMock).toHaveBeenCalledWith(
      "play_path",
      expect.objectContaining({ path: "/a", title: "A" }),
    );
  });

  it("stop clears session, queue, and overlays", async () => {
    invokeMock.mockResolvedValue(undefined);
    const store = usePlaybackStore();
    store.session = session();
    store.playing = true;
    store.coverPath = "/art.jpg";
    store.setQueue([{ path: "/a", title: "A", isVideo: true }], 0);
    store.tracksOpen = true;
    store.eqOpen = true;

    await store.stop();

    expect(store.session.active).toBe(false);
    expect(store.playing).toBe(false);
    expect(store.coverPath).toBeNull();
    expect(store.queue).toEqual([]);
    expect(store.queueIndex).toBe(-1);
    expect(store.tracksOpen).toBe(false);
    expect(store.eqOpen).toBe(false);
    expect(invokeMock).toHaveBeenCalledWith("stop_playback");
  });

  it("showNowBar is true only with active session off the player route", () => {
    const store = usePlaybackStore();
    useUiStore();

    store.session = session({ active: true });
    mockRouterState().currentRoute.value = { name: "home", path: "/" };
    expect(store.showNowBar).toBe(true);

    mockRouterState().currentRoute.value = { name: "player", path: "/player" };
    expect(store.showNowBar).toBe(false);
  });
});
