<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AppIcon from "../components/AppIcon.vue";

const { t } = useI18n();

const rows: { action: string; keys: string }[] = [
  { action: "shortcuts.playPause", keys: "Space" },
  { action: "shortcuts.stop", keys: "S" },
  { action: "shortcuts.previous", keys: "P" },
  { action: "shortcuts.next", keys: "N" },
  { action: "shortcuts.seekBack", keys: "← / Shift+← / Alt+←" },
  { action: "shortcuts.seekFwd", keys: "→ / Shift+→ / Alt+→" },
  { action: "shortcuts.volume", keys: "↑ / ↓" },
  { action: "shortcuts.fullscreen", keys: "F11 / F" },
  { action: "shortcuts.hideUi", keys: "Ctrl + \\" },
  { action: "shortcuts.openFiles", keys: "Ctrl + O" },
  { action: "shortcuts.openFolder", keys: "Ctrl + F" },
  { action: "shortcuts.openStream", keys: "Ctrl + N" },
  { action: "shortcuts.settings", keys: "Ctrl + P" },
  { action: "shortcuts.about", keys: "Shift + F1" },
  { action: "shortcuts.quit", keys: "Ctrl + Q" },
  { action: "shortcuts.sidebar", keys: "[" },
];

async function close() {
  try {
    await getCurrentWindow().close();
  } catch {
    /* ignore */
  }
}
</script>

<template>
  <div class="shortcuts">
    <header class="shortcuts-bar" data-tauri-drag-region>
      <span class="shortcuts-title">{{ t("menu.shortcuts") }}</span>
      <button
        type="button"
        class="icon-btn win-close"
        :aria-label="t('common.close')"
        @click="close"
      >
        <AppIcon name="close" :size="14" />
      </button>
    </header>
    <div class="shortcuts-body">
      <div v-for="row in rows" :key="row.action" class="shortcut-row">
        <span class="shortcut-action">{{ t(row.action) }}</span>
        <span class="shortcut-keys">{{ row.keys }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.shortcuts {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  color: var(--text);
}
.shortcuts-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 8px 0 16px;
  border-bottom: 1px solid var(--border);
  flex: none;
  -webkit-app-region: drag;
}
.shortcuts-bar button {
  -webkit-app-region: no-drag;
}
.shortcuts-title {
  font-size: 13px;
  color: var(--muted);
}
.shortcuts-body {
  flex: 1;
  overflow: auto;
  padding: 12px 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.shortcut-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 270px;
  gap: 12px;
  align-items: center;
  min-height: 33px;
}
.shortcut-action {
  font-size: 14px;
  color: var(--muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.shortcut-keys {
  height: 33px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-radius: 6px;
  background: var(--seek-track);
  font-size: 13px;
  color: var(--muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.win-close:hover {
  background: var(--close-hover-bg) !important;
  color: var(--close-hover-ink) !important;
}
</style>
