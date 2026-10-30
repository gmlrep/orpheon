<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import PopupMenu, { type PopupMenuRow } from "../components/PopupMenu.vue";
import { CTX_ACTION_EVENT } from "../utils/popupWindows";

const params = new URLSearchParams(window.location.search);
const playing = params.get("playing") === "1";
const active = params.get("active") === "1";
const isVideo = params.get("isVideo") === "1";

const rows = computed<PopupMenuRow[]>(() => [
  {
    type: "item",
    id: "play-pause",
    labelKey: "player.play",
    icon: "play",
    shortcut: "Space",
    disabled: !active,
  },
  {
    type: "item",
    id: "stop",
    labelKey: "common.stop",
    icon: "stop",
    shortcut: "S",
    disabled: !active,
  },
  {
    type: "item",
    id: "previous",
    labelKey: "player.previous",
    icon: "previous",
    shortcut: "P",
    disabled: !active,
  },
  {
    type: "item",
    id: "next",
    labelKey: "player.next",
    icon: "next",
    shortcut: "N",
    disabled: !active,
  },
  {
    type: "item",
    id: "record",
    labelKey: "menu.record",
    icon: "replay",
    shortcut: "R",
  },
  { type: "sep" },
  {
    type: "item",
    id: "hide-ui",
    labelKey: "menu.hideUi",
    icon: "fullscreen",
    shortcut: "Ctrl + \\",
  },
  { type: "sep" },
  { type: "item", id: "playback", labelKey: "player.playback", icon: "eq" },
  {
    type: "item",
    id: "video",
    labelKey: "player.video",
    icon: "video",
    disabled: !active || !isVideo,
  },
  {
    type: "item",
    id: "audio",
    labelKey: "player.audio",
    icon: "audio",
    disabled: !active,
  },
  {
    type: "item",
    id: "subs",
    labelKey: "player.subs",
    icon: "subtitles",
    disabled: !active || !isVideo,
  },
  { type: "sep" },
  { type: "item", id: "settings", labelKey: "nav.settings", icon: "settings" },
  { type: "item", id: "playlist", labelKey: "nav.playlists", icon: "playlist" },
  { type: "item", id: "open-file", labelKey: "menu.openFiles", icon: "plus" },
  { type: "item", id: "open-folder", labelKey: "menu.openFolder", icon: "folder" },
  { type: "item", id: "open-stream", labelKey: "menu.openStream", icon: "streaming" },
  { type: "item", id: "open-disc", labelKey: "menu.openDisc", icon: "replay" },
  { type: "item", id: "quit", labelKey: "menu.quit", icon: "close", shortcut: "Ctrl + Q" },
]);

async function closeSelf() {
  try {
    await getCurrentWindow().close();
  } catch {
    /* ignore */
  }
}

async function onPick(id: string) {
  try {
    await emit(CTX_ACTION_EVENT, id);
  } catch {
    /* ignore */
  }
  await closeSelf();
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    void closeSelf();
  }
}

let unFocus: (() => void) | undefined;

onMounted(async () => {
  document.documentElement.dataset.popup = "context";
  window.addEventListener("keydown", onKey);
  try {
    unFocus = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (!focused) void closeSelf();
    });
  } catch {
    /* ignore */
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKey);
  unFocus?.();
});
</script>

<template>
  <div class="ctx-root" data-context-menu>
    <PopupMenu :rows="rows" :playing="playing" @pick="onPick" />
  </div>
</template>

<style scoped>
.ctx-root {
  height: 100%;
  padding: 0;
  background: var(--raised);
}
</style>
