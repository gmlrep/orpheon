<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AppIcon from "./AppIcon.vue";

const { t } = useI18n();
const win = getCurrentWindow();

async function minimize() {
  await win.minimize();
}
async function toggleMax() {
  await win.toggleMaximize();
}
async function close() {
  await win.close();
}
</script>

<template>
  <div class="win-btns">
    <button type="button" class="icon-btn" :aria-label="t('shell.minimize')" @click="minimize">
      <AppIcon name="minimize" :size="14" />
    </button>
    <button type="button" class="icon-btn" :aria-label="t('shell.maximize')" @click="toggleMax">
      <AppIcon name="maximize" :size="14" />
    </button>
    <button type="button" class="icon-btn win-close" :aria-label="t('shell.close')" @click="close">
      <AppIcon name="close" :size="14" />
    </button>
  </div>
</template>

<style scoped>
.win-btns {
  display: flex;
  align-items: center;
  gap: 2px;
  flex: none;
  height: 32px;
}
.win-close:hover {
  background: var(--close-hover-bg) !important;
  color: var(--close-hover-ink) !important;
}
</style>
