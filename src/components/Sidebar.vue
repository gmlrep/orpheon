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
  <aside class="sidebar" :aria-label="t('nav.browse')">
    <div class="sidebar-inner">
      <div class="nav-group">{{ t("nav.browse") }}</div>
      <nav class="nav">
        <a
          v-for="item in browseTabs"
          :key="item.id"
          href="#"
          :aria-current="route === item.id ? 'page' : undefined"
          @click.prevent="ui.navigate(item.id)"
        >
          <AppIcon
            :name="item.icon"
            :variant="route === item.id ? 'active' : 'default'"
            :size="16"
          />
          <span class="nav-label">{{ t(item.label) }}</span>
        </a>
      </nav>

      <div class="nav-group">{{ t("nav.open") }}</div>
      <nav class="nav">
        <a
          href="#"
          :aria-current="route === 'stream' ? 'page' : undefined"
          @click.prevent="ui.navigate('stream')"
        >
          <AppIcon name="streaming" :size="16" />
          <span class="nav-label">{{ t("nav.stream") }}</span>
        </a>
      </nav>

      <div class="bottom">
        <nav class="nav">
          <a
            href="#"
            :aria-current="route === 'settings' ? 'page' : undefined"
            @click.prevent="ui.navigate('settings')"
          >
            <AppIcon
              name="settings"
              :variant="route === 'settings' ? 'active' : 'default'"
              :size="16"
            />
            <span class="nav-label">{{ t("nav.settings") }}</span>
          </a>
        </nav>
      </div>
    </div>
  </aside>
</template>
