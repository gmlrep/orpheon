<script setup lang="ts">
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";
import { useUiStore } from "../stores/ui";
import type { RouteName } from "../types";
import AppIcon from "./AppIcon.vue";

const { t } = useI18n();
const ui = useUiStore();
const { route } = storeToRefs(ui);

const tabs: { id: RouteName; icon: string; label: string }[] = [
  { id: "home", icon: "home", label: "nav.home" },
  { id: "videos", icon: "video", label: "nav.videos" },
  { id: "music", icon: "music", label: "nav.music" },
  { id: "playlists", icon: "playlist", label: "nav.playlists" },
  { id: "recent", icon: "recent", label: "nav.recent" },
];
</script>

<template>
  <nav class="phone-tabs" aria-label="Primary">
    <a
      v-for="tab in tabs"
      :key="tab.id"
      href="#"
      :aria-current="route === tab.id ? 'page' : undefined"
      @click.prevent="ui.navigate(tab.id)"
    >
      <AppIcon :name="tab.icon" :size="18" />
      <span>{{ t(tab.label) }}</span>
    </a>
  </nav>
</template>
