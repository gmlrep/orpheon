<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AppIcon from "../components/AppIcon.vue";

const { t } = useI18n();

async function close() {
  try {
    await getCurrentWindow().close();
  } catch {
    /* ignore */
  }
}
</script>

<template>
  <div class="customize">
    <header class="customize-bar" data-tauri-drag-region>
      <span class="customize-title">{{ t("menu.customize") }}</span>
      <button
        type="button"
        class="icon-btn win-close"
        :aria-label="t('common.close')"
        @click="close"
      >
        <AppIcon name="close" :size="14" />
      </button>
    </header>
    <div class="customize-body">
      <p>{{ t("menu.comingSoonDetail") }}</p>
    </div>
  </div>
</template>

<style scoped>
.customize {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  color: var(--text);
}
.customize-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 8px 0 16px;
  border-bottom: 1px solid var(--border);
  flex: none;
  -webkit-app-region: drag;
}
.customize-bar button {
  -webkit-app-region: no-drag;
}
.customize-title {
  font-size: 13px;
  color: var(--muted);
}
.customize-body {
  flex: 1;
  display: grid;
  place-items: center;
  padding: 32px;
  color: var(--muted);
  font-size: 14px;
  text-align: center;
}
.win-close:hover {
  background: var(--close-hover-bg) !important;
  color: var(--close-hover-ink) !important;
}
</style>
