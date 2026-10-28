<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { storeToRefs } from "pinia";
import { useSettingsStore } from "../stores/settings";
import { useUiStore } from "../stores/ui";
import { setLocale } from "../i18n";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const ui = useUiStore();
const { settings } = storeToRefs(settingsStore);
const { settingsCat } = storeToRefs(ui);

onMounted(() => {
  void settingsStore.load();
});

const nav = computed(() => [
  {
    section: "App",
    items: [
      { id: "general", label: t("settings.general") },
      { id: "interface", label: t("settings.interface") },
      { id: "privacy", label: t("settings.privacy") },
      { id: "controls", label: t("settings.controls") },
    ],
  },
  {
    section: "Media",
    items: [
      { id: "playback", label: t("settings.playback") },
      { id: "video", label: t("settings.video") },
      { id: "audio", label: t("settings.audio") },
      { id: "subtitles", label: t("settings.subtitles") },
      { id: "library", label: t("settings.library") },
    ],
  },
  {
    section: "System",
    items: [
      { id: "network", label: t("settings.network") },
      { id: "performance", label: t("settings.performance") },
      { id: "recording", label: t("settings.recording") },
      { id: "extensions", label: t("settings.extensions") },
    ],
  },
]);

async function toggleResume() {
  await settingsStore.save({ resume_playback: !settings.value.resume_playback });
}
async function toggleHw() {
  await settingsStore.save({ hw_accel: !settings.value.hw_accel });
}
async function onTheme(theme: "system" | "light" | "dark") {
  await settingsStore.setTheme(theme);
}
async function onLang(language: string) {
  await settingsStore.setLanguage(language);
  setLocale(language === "system" ? navigator.language : language);
}
</script>

<template>
  <div>
    <div class="title-row">
      <h1>{{ t("settings.title") }}</h1>
    </div>
    <div class="settings-layout">
      <nav class="settings-nav" aria-label="Settings">
        <template v-for="group in nav" :key="group.section">
          <div class="group">{{ group.section }}</div>
          <button
            v-for="item in group.items"
            :key="item.id"
            type="button"
            :aria-current="settingsCat === item.id"
            @click="settingsCat = item.id"
          >
            {{ item.label }}
          </button>
        </template>
      </nav>

      <div class="settings-page">
        <h2>{{ settingsCat }}</h2>

        <template v-if="settingsCat === 'general' || settingsCat === 'interface'">
          <div class="settings-row">
            <div class="label">
              <strong>{{ t("settings.theme") }}</strong>
              <span>Appearance</span>
            </div>
            <div class="seg">
              <button
                type="button"
                :aria-pressed="settings.theme === 'system'"
                @click="onTheme('system')"
              >
                System
              </button>
              <button
                type="button"
                :aria-pressed="settings.theme === 'dark'"
                @click="onTheme('dark')"
              >
                Dark
              </button>
              <button
                type="button"
                :aria-pressed="settings.theme === 'light'"
                @click="onTheme('light')"
              >
                Light
              </button>
            </div>
          </div>
          <div class="settings-row">
            <div class="label">
              <strong>{{ t("settings.language") }}</strong>
            </div>
            <div class="seg">
              <button
                type="button"
                :aria-pressed="settings.language === 'system'"
                @click="onLang('system')"
              >
                System
              </button>
              <button
                type="button"
                :aria-pressed="settings.language === 'en'"
                @click="onLang('en')"
              >
                English
              </button>
              <button
                type="button"
                :aria-pressed="settings.language === 'ru'"
                @click="onLang('ru')"
              >
                Русский
              </button>
            </div>
          </div>
        </template>

        <template v-if="settingsCat === 'playback' || settingsCat === 'general'">
          <div class="settings-row">
            <div class="label">
              <strong>{{ t("settings.resume") }}</strong>
            </div>
            <button
              type="button"
              class="switch"
              role="switch"
              :aria-checked="settings.resume_playback"
              @click="toggleResume"
            >
              <i />
            </button>
          </div>
        </template>

        <template v-if="settingsCat === 'performance' || settingsCat === 'video'">
          <div class="settings-row">
            <div class="label">
              <strong>{{ t("settings.hwAccel") }}</strong>
            </div>
            <button
              type="button"
              class="switch"
              role="switch"
              :aria-checked="settings.hw_accel"
              @click="toggleHw"
            >
              <i />
            </button>
          </div>
        </template>

        <template v-if="settingsCat === 'audio'">
          <div class="settings-row">
            <div class="label">
              <strong>{{ t("settings.volume") }}</strong>
              <span>{{ settings.volume }}%</span>
            </div>
            <input
              type="range"
              min="0"
              max="200"
              :value="settings.volume"
              @change="
                settingsStore.save({
                  volume: Number(($event.target as HTMLInputElement).value),
                })
              "
            />
          </div>
        </template>

        <p v-if="!['general', 'interface', 'playback', 'video', 'audio', 'performance'].includes(settingsCat)" class="empty">
          Preferences for this section are stored with the app and will expand as features land.
        </p>
      </div>
    </div>
  </div>
</template>
