<script setup lang="ts">
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";
import { useUiStore } from "../stores/ui";
import markUrl from "../assets/mark.svg";
import NowBar from "./NowBar.vue";
import WindowControls from "./WindowControls.vue";

const { t } = useI18n();
const ui = useUiStore();
const { sideToggleState, fullscreen } = storeToRefs(ui);
</script>

<template>
  <header v-show="!fullscreen" class="topbar">
    <div class="topbar-left">
      <span class="brand-mark" aria-hidden="true">
        <img :src="markUrl" alt="" width="22" height="22" />
      </span>
      <span class="app-title">{{ t("app.name") }}</span>
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
        <svg class="side-toggle-frame" viewBox="0 0 28 28" width="28" height="28" aria-hidden="true">
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
