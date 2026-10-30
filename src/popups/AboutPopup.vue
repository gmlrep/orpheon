<script setup lang="ts">
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { getName, getTauriVersion, getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import markUrl from "../assets/mark.svg";
import AppIcon from "../components/AppIcon.vue";

const { t } = useI18n();
const version = shallowRef("…");
const tauriVersion = shallowRef("…");
const appName = shallowRef("Orpheon");

onMounted(async () => {
  try {
    const [v, tv, name] = await Promise.all([getVersion(), getTauriVersion(), getName()]);
    version.value = v;
    tauriVersion.value = tv;
    appName.value = name;
  } catch {
    version.value = "0.1.1";
  }
});

async function close() {
  try {
    await getCurrentWindow().close();
  } catch {
    /* ignore */
  }
}
</script>

<template>
  <div class="about">
    <header class="about-bar" data-tauri-drag-region>
      <span class="about-title">{{ t("menu.about") }}</span>
      <button
        type="button"
        class="icon-btn win-close"
        :aria-label="t('common.close')"
        @click="close"
      >
        <AppIcon name="close" :size="14" />
      </button>
    </header>
    <div class="about-body">
      <img class="about-mark" :src="markUrl" alt="" width="72" height="72" />
      <h1>{{ appName }}</h1>
      <p class="about-version">{{ t("about.version", { version }) }}</p>
      <p class="about-desc">{{ t("about.description") }}</p>
      <p class="about-meta">{{ t("about.tauri", { version: tauriVersion }) }}</p>
      <p class="about-license">{{ t("about.license") }}</p>
    </div>
  </div>
</template>

<style scoped>
.about {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  color: var(--text);
}
.about-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 8px 0 16px;
  border-bottom: 1px solid var(--border);
  flex: none;
  -webkit-app-region: drag;
}
.about-bar button {
  -webkit-app-region: no-drag;
}
.about-title {
  font-size: 13px;
  color: var(--muted);
}
.about-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
  padding: 28px 32px 32px;
  overflow: auto;
}
.about-mark {
  border-radius: 14px;
  margin-bottom: 4px;
}
h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
}
.about-version {
  margin: 0;
  color: var(--accent);
  font-weight: 600;
  font-size: 14px;
}
.about-desc {
  margin: 8px 0 0;
  color: var(--muted);
  font-size: 14px;
  line-height: 1.45;
  white-space: pre-line;
  max-width: 42ch;
}
.about-meta {
  margin: 4px 0 0;
  color: var(--faint);
  font-size: 12px;
}
.about-license {
  margin: 12px 0 0;
  color: var(--faint);
  font-size: 12px;
  line-height: 1.4;
  white-space: pre-line;
}
.win-close:hover {
  background: var(--close-hover-bg) !important;
  color: var(--close-hover-ink) !important;
}
</style>
