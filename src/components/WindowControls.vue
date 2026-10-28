<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import AppIcon from "./AppIcon.vue";

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
    <button type="button" class="icon-btn" aria-label="Minimize" @click="minimize">
      <AppIcon name="minimize" :size="14" />
    </button>
    <button type="button" class="icon-btn" aria-label="Maximize" @click="toggleMax">
      <AppIcon name="maximize" :size="14" />
    </button>
    <button type="button" class="icon-btn win-close" aria-label="Close" @click="close">
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
  background: #e5566f !important;
  color: #fff !important;
}
</style>
