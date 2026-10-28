import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { RouteName, SidebarMode } from "../types";

export const useUiStore = defineStore("ui", () => {
  const sidebarMode = ref<SidebarMode>("open");
  const route = ref<RouteName>("home");
  const settingsCat = ref("general");
  const listQuery = ref("");
  const videoSort = ref<"title" | "added" | "duration" | "recent">("added");
  const musicSort = ref<"title" | "artist" | "album" | "duration">("title");
  const tracksOpen = ref(false);
  const trackTab = ref<"video" | "audio" | "subs">("video");
  const eqOpen = ref(false);
  const fullscreen = ref(false);
  const uiHidden = ref(false);
  /** Sidebar mode to restore after leaving fullscreen. */
  let sidebarBeforeFullscreen: SidebarMode | null = null;

  /** i18n key for the topbar crumb (resolved in TopBar). */
  const crumbKey = computed(() => {
    const map: Record<RouteName, string> = {
      home: "nav.home",
      videos: "nav.videos",
      music: "nav.music",
      playlists: "nav.playlists",
      recent: "nav.recent",
      stream: "nav.stream",
      settings: "nav.settings",
      player: "player.playing",
      eq: "eq.title",
    };
    return map[route.value];
  });
  /** @deprecated use crumbKey + i18n */
  const crumb = crumbKey;

  const sideToggleState = computed(() => {
    if (sidebarMode.value === "open" || sidebarMode.value === "sheet") return "max";
    if (sidebarMode.value === "rail") return "min";
    return "none";
  });

  function cycleSidebar() {
    const narrow = window.matchMedia("(max-width: 900px)").matches;
    if (narrow) {
      sidebarMode.value = sidebarMode.value === "sheet" ? "hidden" : "sheet";
      return;
    }
    const order: SidebarMode[] = ["open", "rail", "hidden"];
    const i = order.indexOf(sidebarMode.value);
    sidebarMode.value = order[(i + 1) % order.length];
  }

  /** Navigate without changing the user's chosen sidebar mode. */
  function navigate(name: RouteName) {
    // Auto-narrow only when entering the player from a labelled sidebar.
    if (name === "player" && route.value !== "player" && sidebarMode.value === "open") {
      sidebarMode.value = "rail";
    }
    // Leaving player / switching library sections must NOT force-expand the sidebar.
    route.value = name;
    if (name !== "player") {
      tracksOpen.value = false;
      eqOpen.value = false;
    }
  }

  function openTrackTab(tab: "video" | "audio" | "subs") {
    if (tracksOpen.value && trackTab.value === tab) {
      tracksOpen.value = false;
      return;
    }
    trackTab.value = tab;
    tracksOpen.value = true;
    eqOpen.value = false;
  }

  function setFullscreen(value: boolean) {
    if (value === fullscreen.value) return;
    fullscreen.value = value;
    if (value) {
      sidebarBeforeFullscreen = sidebarMode.value;
      sidebarMode.value = "hidden";
      uiHidden.value = false;
    } else {
      uiHidden.value = false;
      // Restore pre-FS mode; never force-expand past what the user had.
      const restore = sidebarBeforeFullscreen ?? "rail";
      sidebarBeforeFullscreen = null;
      sidebarMode.value = restore === "hidden" ? "rail" : restore;
    }
  }

  return {
    sidebarMode,
    route,
    settingsCat,
    listQuery,
    videoSort,
    musicSort,
    tracksOpen,
    trackTab,
    eqOpen,
    fullscreen,
    uiHidden,
    crumb,
    crumbKey,
    sideToggleState,
    cycleSidebar,
    navigate,
    openTrackTab,
    setFullscreen,
  };
});
