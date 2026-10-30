import { onMounted, onUnmounted, shallowRef } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "vue-i18n";
import { usePlaybackStore } from "../stores/playback";
import { useLibraryStore } from "../stores/library";
import { useUiStore } from "../stores/ui";
import {
  CTX_ACTION_EVENT,
  closeContextMenuWindow,
  openAboutWindow,
  openContextMenuWindow,
  openCustomizeWindow,
  openShortcutsWindow,
} from "../utils/popupWindows";

const MEDIA_FILTERS = [
  {
    name: "Media",
    extensions: [
      "mkv",
      "mp4",
      "avi",
      "mov",
      "webm",
      "m4v",
      "wmv",
      "flv",
      "ts",
      "m2ts",
      "mp3",
      "flac",
      "m4a",
      "aac",
      "wav",
      "ogg",
      "opus",
      "wma",
    ],
  },
];

export const PROFILE_IDS = ["Movies", "Shows", "Music", "Work", "New"] as const;
export type ProfileId = (typeof PROFILE_IDS)[number];

const PROFILE_KEY = "orpheon.activeProfile";

const appOpen = shallowRef(false);
const profileFlyout = shallowRef(false);
const activeProfile = shallowRef<ProfileId>(readProfile());

function readProfile(): ProfileId {
  try {
    const v = localStorage.getItem(PROFILE_KEY);
    if (v && (PROFILE_IDS as readonly string[]).includes(v)) return v as ProfileId;
  } catch {
    /* ignore */
  }
  return "Movies";
}

function writeProfile(id: ProfileId) {
  activeProfile.value = id;
  try {
    localStorage.setItem(PROFILE_KEY, id);
  } catch {
    /* ignore */
  }
}

export function useAppMenus() {
  const playback = usePlaybackStore();
  const library = useLibraryStore();
  const ui = useUiStore();
  const { t } = useI18n();

  function closeAll() {
    appOpen.value = false;
    profileFlyout.value = false;
    void closeContextMenuWindow();
  }

  function toggleAppMenu() {
    void closeContextMenuWindow();
    profileFlyout.value = false;
    appOpen.value = !appOpen.value;
  }

  function flashComingSoon(featureKey: string) {
    ui.flash(t("menu.comingSoon", { feature: t(featureKey) }));
  }

  async function openFiles() {
    closeAll();
    const selected = await open({
      multiple: true,
      directory: false,
      filters: MEDIA_FILTERS,
    });
    const paths = Array.isArray(selected) ? selected : selected ? [selected] : [];
    if (!paths.length) return;
    const queue = paths.map((path) => ({
      path,
      title: path.split(/[/\\]/).pop() ?? path,
      isVideo: !/\.(mp3|flac|m4a|aac|wav|ogg|opus|wma)$/i.test(path),
    }));
    await playback.playPath(queue[0].path, {
      title: queue[0].title,
      isVideo: queue[0].isVideo,
      queue,
      queueIndex: 0,
    });
  }

  async function openFolder() {
    closeAll();
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected !== "string" || !selected) return;
    await library.addFolder(selected, "video");
    ui.navigate("videos");
  }

  function openNetworkStream() {
    closeAll();
    ui.navigate("stream");
  }

  function openRecent() {
    closeAll();
    ui.navigate("recent");
  }

  function openSettings() {
    closeAll();
    ui.navigate("settings");
  }

  function openPlaylists() {
    closeAll();
    ui.navigate("playlists");
  }

  function openEq() {
    closeAll();
    ui.navigate("eq");
  }

  async function openAbout() {
    closeAll();
    try {
      await openAboutWindow();
    } catch (e) {
      console.error(e);
    }
  }

  async function openShortcuts() {
    closeAll();
    try {
      await openShortcutsWindow();
    } catch (e) {
      console.error(e);
    }
  }

  async function openCustomize() {
    closeAll();
    try {
      await openCustomizeWindow();
    } catch (e) {
      console.error(e);
    }
  }

  async function quitApp() {
    closeAll();
    try {
      await getCurrentWindow().close();
    } catch {
      /* ignore */
    }
  }

  async function openContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    appOpen.value = false;
    profileFlyout.value = false;
    try {
      await openContextMenuWindow({
        playing: playback.playing,
        active: playback.session.active,
        isVideo: playback.session.isVideo,
      });
    } catch (err) {
      console.error(err);
    }
  }

  async function runContextAction(id: string) {
    switch (id) {
      case "play-pause":
        closeAll();
        if (playback.session.active) await playback.playPause();
        break;
      case "stop":
        closeAll();
        await playback.stop();
        break;
      case "previous":
        closeAll();
        await playback.previousTrack();
        break;
      case "next":
        closeAll();
        await playback.nextTrack();
        break;
      case "record":
        closeAll();
        flashComingSoon("menu.record");
        break;
      case "hide-ui":
        closeAll();
        ui.toggleUiHidden();
        break;
      case "video":
        closeAll();
        if (playback.session.active) void playback.openPlayer();
        playback.openTrackTab("video");
        break;
      case "audio":
        closeAll();
        if (playback.session.active) void playback.openPlayer();
        playback.openTrackTab("audio");
        break;
      case "subs":
        closeAll();
        if (playback.session.active) void playback.openPlayer();
        playback.openTrackTab("subs");
        break;
      case "playback":
        openEq();
        break;
      case "settings":
        openSettings();
        break;
      case "playlist":
        openPlaylists();
        break;
      case "open-file":
        await openFiles();
        break;
      case "open-folder":
        await openFolder();
        break;
      case "open-stream":
        openNetworkStream();
        break;
      case "open-disc":
        closeAll();
        flashComingSoon("menu.openDisc");
        break;
      case "quit":
        await quitApp();
        break;
      default:
        closeAll();
    }
  }

  async function runAppAction(id: string) {
    if (id.startsWith("profile:")) {
      const name = id.slice("profile:".length) as ProfileId;
      if ((PROFILE_IDS as readonly string[]).includes(name)) {
        writeProfile(name);
        profileFlyout.value = false;
        appOpen.value = false;
      }
      return;
    }

    switch (id) {
      case "open-file":
        await openFiles();
        break;
      case "open-folder":
        await openFolder();
        break;
      case "open-stream":
        openNetworkStream();
        break;
      case "open-disc":
        closeAll();
        flashComingSoon("menu.openDisc");
        break;
      case "open-capture":
        closeAll();
        flashComingSoon("menu.openCapture");
        break;
      case "convert":
        closeAll();
        flashComingSoon("menu.convert");
        break;
      case "open-recent":
        openRecent();
        break;
      case "profile":
        profileFlyout.value = !profileFlyout.value;
        break;
      case "shortcuts":
        await openShortcuts();
        break;
      case "customize":
        await openCustomize();
        break;
      case "settings":
        openSettings();
        break;
      case "about":
        await openAbout();
        break;
      case "quit":
        await quitApp();
        break;
      default:
        closeAll();
    }
  }

  function onDocPointerDown(e: PointerEvent) {
    const el = e.target as HTMLElement | null;
    if (el?.closest?.("[data-app-menu], [data-menu-trigger]")) return;
    appOpen.value = false;
    profileFlyout.value = false;
  }

  function onContextMenu(e: MouseEvent) {
    const el = e.target as HTMLElement | null;
    if (el?.closest?.("input, textarea, [contenteditable='true']")) return;
    void openContextMenu(e);
  }

  function onKey(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA") return;
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.key.toLowerCase() === "o") {
      e.preventDefault();
      void openFiles();
    } else if (mod && e.key.toLowerCase() === "f") {
      e.preventDefault();
      void openFolder();
    } else if (mod && e.key.toLowerCase() === "n") {
      e.preventDefault();
      openNetworkStream();
    } else if (mod && e.key.toLowerCase() === "d") {
      e.preventDefault();
      closeAll();
      flashComingSoon("menu.openDisc");
    } else if (mod && e.key.toLowerCase() === "c") {
      e.preventDefault();
      closeAll();
      flashComingSoon("menu.openCapture");
    } else if (mod && e.key.toLowerCase() === "r") {
      e.preventDefault();
      closeAll();
      flashComingSoon("menu.convert");
    } else if (mod && e.key.toLowerCase() === "p") {
      e.preventDefault();
      openSettings();
    } else if (mod && e.key.toLowerCase() === "q") {
      e.preventDefault();
      void quitApp();
    } else if (e.shiftKey && e.key === "F1") {
      e.preventDefault();
      void openAbout();
    } else if (mod && e.key === "\\") {
      e.preventDefault();
      ui.toggleUiHidden();
    } else if (e.key === "Escape") {
      closeAll();
    }
  }

  /** Call once from App.vue setup to bind global menu listeners. */
  function installListeners() {
    let unCtx: (() => void) | undefined;

    onMounted(() => {
      document.addEventListener("pointerdown", onDocPointerDown, true);
      document.addEventListener("contextmenu", onContextMenu);
      window.addEventListener("keydown", onKey);
      void listen<string>(CTX_ACTION_EVENT, (ev) => {
        void runContextAction(ev.payload);
      }).then((fn) => {
        unCtx = fn;
      });
    });
    onUnmounted(() => {
      document.removeEventListener("pointerdown", onDocPointerDown, true);
      document.removeEventListener("contextmenu", onContextMenu);
      window.removeEventListener("keydown", onKey);
      unCtx?.();
    });
  }

  return {
    appOpen,
    profileFlyout,
    activeProfile,
    toggleAppMenu,
    closeAll,
    runAppAction,
    runContextAction,
    installListeners,
  };
}
