<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from "vue";
import { storeToRefs } from "pinia";
import { getCurrentWindow } from "@tauri-apps/api/window";
import TopBar from "./components/TopBar.vue";
import Sidebar from "./components/Sidebar.vue";
import PhoneTabs from "./components/PhoneTabs.vue";
import HomeView from "./views/HomeView.vue";
import LibraryView from "./views/LibraryView.vue";
import PlayerView from "./views/PlayerView.vue";
import SettingsView from "./views/SettingsView.vue";
import PlaylistsView from "./views/PlaylistsView.vue";
import StreamView from "./views/StreamView.vue";
import RecentView from "./views/RecentView.vue";
import EqView from "./views/EqView.vue";
import { useUiStore } from "./stores/ui";
import { usePlaybackStore } from "./stores/playback";
import { useSettingsStore } from "./stores/settings";
import { setLocale } from "./i18n";

const ui = useUiStore();
const playback = usePlaybackStore();
const settings = useSettingsStore();
const { route, sidebarMode, fullscreen } = storeToRefs(ui);

const screen = computed(() => route.value);

async function toggleFullscreen() {
  const win = getCurrentWindow();
  const next = !(await win.isFullscreen());
  await win.setFullscreen(next);
  ui.setFullscreen(next);
}

function onKey(e: KeyboardEvent) {
  const tag = (e.target as HTMLElement)?.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA") return;

  if (e.key === "[" && !e.ctrlKey && !e.metaKey && !e.altKey) {
    e.preventDefault();
    ui.cycleSidebar();
  }
  if (e.code === "Space") {
    if (playback.session.active) {
      e.preventDefault();
      void playback.playPause();
    }
  }
  if (e.key === "F11" || (e.key === "f" && !e.ctrlKey && !e.metaKey && !e.altKey)) {
    e.preventDefault();
    void toggleFullscreen();
  }
  if (e.key === "Escape") {
    if (ui.fullscreen) {
      e.preventDefault();
      void toggleFullscreen();
    } else if (ui.tracksOpen) {
      ui.tracksOpen = false;
    }
  }
  if (e.key === "MediaTrackNext" || (e.key === "n" && !e.ctrlKey && !e.metaKey)) {
    if (playback.session.active) void playback.nextTrack();
  }
  if (e.key === "MediaTrackPrevious" || (e.key === "p" && !e.ctrlKey && !e.metaKey)) {
    if (playback.session.active) void playback.previousTrack();
  }
}

watch(fullscreen, (fs) => {
  document.documentElement.dataset.fullscreen = fs ? "1" : "0";
});

// Leaving the player (sidebar nav, etc.) must tear down the HWND overlay.
watch(route, async (name, prev) => {
  if (prev === "player" && name !== "player") {
    try {
      await playback.tearDownVideoSurface();
    } catch {
      /* ignore */
    }
  }
});

onMounted(async () => {
  document.documentElement.dataset.theme = "dark";
  await settings.load();
  const lang = settings.settings.language;
  setLocale(lang === "system" ? navigator.language : lang);
  await playback.bootstrap();
  window.addEventListener("keydown", onKey);

  // Sync OS fullscreen → store (e.g. system shortcuts).
  try {
    const win = getCurrentWindow();
    ui.setFullscreen(await win.isFullscreen());
  } catch {
    /* ignore */
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKey);
});
</script>

<template>
  <div
    class="app-window"
    :data-sidebar="sidebarMode"
    :data-screen="screen"
    :data-fullscreen="fullscreen ? 'true' : undefined"
  >
    <TopBar v-if="!fullscreen" />
    <div class="body">
      <Sidebar v-if="!fullscreen" />
      <main class="canvas">
        <Transition name="view" mode="out-in">
          <HomeView v-if="screen === 'home'" :key="'home'" />
          <LibraryView v-else-if="screen === 'videos'" key="videos" kind="video" />
          <LibraryView v-else-if="screen === 'music'" key="music" kind="audio" />
          <PlaylistsView v-else-if="screen === 'playlists'" key="playlists" />
          <RecentView v-else-if="screen === 'recent'" key="recent" />
          <StreamView v-else-if="screen === 'stream'" key="stream" />
          <SettingsView v-else-if="screen === 'settings'" key="settings" />
          <EqView v-else-if="screen === 'eq'" key="eq" />
          <PlayerView v-else-if="screen === 'player'" key="player" />
        </Transition>
      </main>
    </div>
    <PhoneTabs v-if="!fullscreen" />
  </div>
</template>
