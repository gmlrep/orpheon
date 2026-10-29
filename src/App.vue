<script setup lang="ts">
import { onMounted, watch } from "vue";
import { storeToRefs } from "pinia";
import TopBar from "./components/TopBar.vue";
import Sidebar from "./components/Sidebar.vue";
import PhoneTabs from "./components/PhoneTabs.vue";
import { useUiStore } from "./stores/ui";
import { usePlaybackStore } from "./stores/playback";
import { useSettingsStore } from "./stores/settings";
import { useAppHotkeys } from "./composables/useAppHotkeys";
import { useFullscreen } from "./composables/useFullscreen";
import { useWindowPrefs } from "./composables/useWindowPrefs";
import { setLocale } from "./i18n";

const ui = useUiStore();
const playback = usePlaybackStore();
const settings = useSettingsStore();
const { route, sidebarMode, fullscreen } = storeToRefs(ui);
const { syncFromOs } = useFullscreen();

useAppHotkeys();
useWindowPrefs();

const keepAliveIncludes = ["HomeView", "LibraryView", "PlaylistsView", "RecentView"];

watch(fullscreen, (fs) => {
  document.documentElement.dataset.fullscreen = fs ? "1" : "0";
});

onMounted(async () => {
  await settings.load();
  const lang = settings.settings.language;
  setLocale(lang === "system" ? navigator.language : lang);
  await playback.bootstrap();
  await syncFromOs();
});
</script>

<template>
  <div
    class="app-window"
    :data-sidebar="sidebarMode"
    :data-screen="route"
    :data-fullscreen="fullscreen ? 'true' : undefined"
  >
    <TopBar v-if="!fullscreen" />
    <div class="body">
      <Sidebar v-if="!fullscreen" />
      <main class="canvas">
        <RouterView v-slot="{ Component, route: r }">
          <Transition name="view" mode="out-in">
            <KeepAlive :include="keepAliveIncludes">
              <component :is="Component" :key="String(r.name)" />
            </KeepAlive>
          </Transition>
        </RouterView>
      </main>
    </div>
    <PhoneTabs v-if="!fullscreen" />
  </div>
</template>
