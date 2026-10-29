<script setup lang="ts">
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";
import { useUiStore } from "../stores/ui";
import { browseTabs } from "../nav";
import AppIcon from "./AppIcon.vue";

const { t } = useI18n();
const ui = useUiStore();
const { route } = storeToRefs(ui);
</script>

<template>
  <nav class="phone-tabs" :aria-label="t('nav.primary')">
    <a
      v-for="tab in browseTabs"
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
