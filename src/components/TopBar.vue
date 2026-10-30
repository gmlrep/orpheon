<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";
import { useUiStore } from "../stores/ui";
import { PROFILE_IDS, useAppMenus } from "../composables/useAppMenus";
import markUrl from "../assets/mark.svg";
import NowBar from "./NowBar.vue";
import WindowControls from "./WindowControls.vue";
import PopupMenu, { type PopupMenuRow } from "./PopupMenu.vue";
import AppIcon from "./AppIcon.vue";

const { t } = useI18n();
const ui = useUiStore();
const { sideToggleState, fullscreen } = storeToRefs(ui);
const menus = useAppMenus();
const { appOpen, profileFlyout, activeProfile, toggleAppMenu, runAppAction } = menus;

const profileLabel = computed(() =>
  t("menu.profile", { name: t(`profiles.${activeProfile.value}`) }),
);

const appMenuRows = computed<PopupMenuRow[]>(() => [
  { type: "item", id: "open-file", labelKey: "menu.openFiles", icon: "plus", shortcut: "Ctrl + O" },
  {
    type: "item",
    id: "open-folder",
    labelKey: "menu.openFolder",
    icon: "folder",
    shortcut: "Ctrl + F",
  },
  {
    type: "item",
    id: "open-stream",
    labelKey: "menu.openStream",
    icon: "streaming",
    shortcut: "Ctrl + N",
  },
  {
    type: "item",
    id: "open-disc",
    labelKey: "menu.openDisc",
    icon: "replay",
    shortcut: "Ctrl + D",
  },
  {
    type: "item",
    id: "open-capture",
    labelKey: "menu.openCapture",
    icon: "video",
    shortcut: "Ctrl + C",
  },
  {
    type: "item",
    id: "convert",
    labelKey: "menu.convert",
    icon: "refresh",
    shortcut: "Ctrl + R",
  },
  {
    type: "item",
    id: "open-recent",
    labelKey: "menu.openRecent",
    icon: "recent",
  },
  { type: "sep" },
  {
    type: "item",
    id: "profile",
    labelKey: "menu.profile",
    label: profileLabel.value,
    icon: "lists",
  },
  {
    type: "item",
    id: "shortcuts",
    labelKey: "menu.shortcuts",
    icon: "hotkeys",
  },
  {
    type: "item",
    id: "customize",
    labelKey: "menu.customize",
    icon: "playback-settings",
  },
  {
    type: "item",
    id: "settings",
    labelKey: "nav.settings",
    icon: "settings",
    shortcut: "Ctrl + P",
  },
  {
    type: "item",
    id: "about",
    labelKey: "menu.about",
    icon: "info",
    shortcut: "Shift + F1",
  },
]);

const profileRows = computed<PopupMenuRow[]>(() =>
  PROFILE_IDS.map((id) => ({
    type: "item" as const,
    id: `profile:${id}`,
    labelKey: `profiles.${id}`,
    icon: id === activeProfile.value ? "playback" : "lists",
  })),
);
</script>

<template>
  <header v-show="!fullscreen" class="topbar">
    <div class="topbar-left">
      <span class="brand-mark" aria-hidden="true">
        <img :src="markUrl" alt="" width="22" height="22" />
      </span>
      <div class="menu-anchor">
        <button
          type="button"
          class="icon-btn menu-trigger"
          data-menu-trigger
          :aria-expanded="appOpen"
          :aria-label="t('menu.open')"
          @click="toggleAppMenu()"
        >
          <AppIcon name="more" :size="18" />
        </button>
        <div v-if="appOpen" class="menu-flyout" data-app-menu>
          <PopupMenu :rows="appMenuRows" @pick="runAppAction" />
          <div v-if="profileFlyout" class="profile-flyout">
            <PopupMenu :rows="profileRows" @pick="runAppAction" />
          </div>
        </div>
      </div>
      <button
        type="button"
        class="side-toggle"
        :data-state="sideToggleState"
        :aria-expanded="sideToggleState === 'max'"
        :aria-label="
          sideToggleState === 'max'
            ? t('shell.collapseSidebar')
            : sideToggleState === 'min'
              ? t('shell.hideSidebar')
              : t('shell.showSidebar')
        "
        @click="ui.cycleSidebar()"
      >
        <svg
          class="side-toggle-frame"
          viewBox="0 0 28 28"
          width="28"
          height="28"
          aria-hidden="true"
        >
          <path
            d="M8 23H20C21.6569 23 23 21.6569 23 20V8C23 6.34315 21.6569 5 20 5H8C6.34315 5 5 6.34315 5 8V20C5 21.6569 6.34315 23 8 23Z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          />
        </svg>
        <span class="side-toggle-rail" aria-hidden="true" />
      </button>
    </div>
    <div class="topbar-center">
      <NowBar />
    </div>
    <div class="topbar-right">
      <WindowControls />
    </div>
  </header>
</template>

<style scoped>
.menu-anchor {
  position: relative;
}
.menu-trigger {
  color: var(--muted);
}
.menu-trigger:hover,
.menu-trigger[aria-expanded="true"] {
  color: var(--text);
}
.menu-flyout {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  z-index: 80;
  display: flex;
  align-items: flex-start;
  gap: 4px;
}
.profile-flyout {
  margin-top: 356px;
}
</style>
