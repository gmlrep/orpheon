<script setup lang="ts">
import { computed, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { storeToRefs } from "pinia";
import { useSettingsStore } from "../stores/settings";
import { useUiStore } from "../stores/ui";
import { setLocale } from "../i18n";
import {
  SETTINGS_NAV,
  SETTINGS_PAGES,
  pageTitleKey,
  type SettingsCatId,
  type SettingsRow,
} from "../settings/catalog";
import { PREF_OPTIONS } from "../settings/prefOptions";
import type { AppSelectOption } from "../components/appSelectTypes";
import AppSelect from "../components/AppSelect.vue";
import AppSwitch from "../components/AppSwitch.vue";
import AccentPicker from "../components/AccentPicker.vue";
import GhostButton from "../components/GhostButton.vue";
import SettingsTextField from "../components/SettingsTextField.vue";
import PathPicker from "../components/PathPicker.vue";
import PrefsRow from "../components/PrefsRow.vue";

defineOptions({ name: "SettingsView" });

const { t, te } = useI18n();
const settingsStore = useSettingsStore();
const ui = useUiStore();
const { settings } = storeToRefs(settingsStore);
const { settingsCat } = storeToRefs(ui);
const advancedOpen = shallowRef(false);

const themeOptions = computed<AppSelectOption<string>[]>(() => [
  { value: "dark", label: t("settings.themeDark") },
  { value: "light", label: t("settings.themeLight") },
  { value: "system", label: t("settings.themeSystem") },
]);

const languageOptions = computed<AppSelectOption<string>[]>(() => [
  { value: "system", label: t("settings.langSystem") },
  { value: "en", label: t("settings.langEn") },
  { value: "ru", label: t("settings.langRu") },
]);

onMounted(() => {
  void settingsStore.load();
});

const page = computed(
  () => SETTINGS_PAGES[settingsCat.value as SettingsCatId] ?? SETTINGS_PAGES.general,
);

const visibleSections = computed(() =>
  page.value.sections.filter((s) => !s.advanced || advancedOpen.value),
);

const hasAdvanced = computed(() => page.value.sections.some((s) => s.advanced));

function rowLabel(row: SettingsRow): string {
  if (row.type === "theme") return t("settings.theme");
  if (row.type === "language") return t("settings.language");
  if (row.type === "accent") return t("settings.accent");
  if ("key" in row && row.key) {
    const id = `settings.pref.${row.key}`;
    if (te(id)) return t(id);
  }
  return row.label;
}

function rowValue(row: SettingsRow): string | boolean | number {
  if (row.type === "theme") return settings.value.theme;
  if (row.type === "language") return settings.value.language;
  if (row.type === "accent") return settingsStore.prefNumber("accent_index", 0);
  if (row.type === "toggle") {
    if (row.key === "resume_playback") return settings.value.resume_playback;
    if (row.key === "hw_accel") return settings.value.hw_accel;
    return settingsStore.prefBool(row.key);
  }
  if (row.type === "select" || row.type === "text" || row.type === "path") {
    const fallback = row.type === "text" || row.type === "path" ? (row.placeholder ?? "") : "";
    return settingsStore.prefString(row.key, fallback);
  }
  return "";
}

function selectOptions(row: Extract<SettingsRow, { type: "select" }>): AppSelectOption<string>[] {
  const raw = row.options?.length ? row.options : (PREF_OPTIONS[row.key] ?? []);
  return raw.map((value) => ({ value, label: value }));
}

async function onTheme(theme: string) {
  await settingsStore.setTheme(theme as "system" | "light" | "dark");
}

async function onLang(language: string) {
  await settingsStore.setLanguage(language);
  setLocale(language === "system" ? navigator.language : language);
}

async function onToggle(row: Extract<SettingsRow, { type: "toggle" }>, value: boolean) {
  await settingsStore.setPref(row.key, value);
}

async function onSelect(row: Extract<SettingsRow, { type: "select" }>, value: string) {
  await settingsStore.setPref(row.key, value);
}

async function onText(row: Extract<SettingsRow, { type: "text" | "path" }>, value: string) {
  await settingsStore.setPref(row.key, value);
}

async function onAccent(index: number) {
  await settingsStore.setPref("accent_index", index);
}

async function onAction(action: string) {
  await settingsStore.runAction(action);
}
</script>

<template>
  <div>
    <div class="title-row">
      <h1>{{ t("settings.title") }}</h1>
    </div>
    <div class="settings-layout">
      <nav class="settings-nav" :aria-label="t('settings.title')">
        <template v-for="group in SETTINGS_NAV" :key="group.sectionKey">
          <div class="group">{{ t(group.sectionKey) }}</div>
          <button
            v-for="item in group.items"
            :key="item.id"
            type="button"
            :aria-current="settingsCat === item.id"
            @click="ui.setSettingsCat(item.id)"
          >
            {{ t(item.labelKey) }}
          </button>
        </template>
      </nav>

      <div class="settings-page">
        <h2>{{ t(pageTitleKey(settingsCat)) }}</h2>

        <template v-for="(section, si) in visibleSections" :key="`${section.title ?? 's'}-${si}`">
          <div class="settings-block" :class="{ 'is-first': si === 0 }">
            <h3 v-if="section.title" class="settings-section">{{ section.title }}</h3>

            <PrefsRow
              v-for="(row, ri) in section.rows"
              :key="`${row.label}-${ri}`"
              :label="rowLabel(row)"
              :hint="row.hint"
            >
              <AppSelect
                v-if="row.type === 'theme'"
                :model-value="settings.theme"
                :options="themeOptions"
                :label="rowLabel(row)"
                @update:model-value="onTheme"
              />

              <AppSelect
                v-else-if="row.type === 'language'"
                :model-value="settings.language"
                :options="languageOptions"
                :label="rowLabel(row)"
                @update:model-value="onLang"
              />

              <AccentPicker
                v-else-if="row.type === 'accent'"
                :model-value="settingsStore.prefNumber('accent_index', 0)"
                :label="rowLabel(row)"
                @update:model-value="onAccent"
              />

              <AppSwitch
                v-else-if="row.type === 'toggle'"
                :model-value="Boolean(rowValue(row))"
                :label="rowLabel(row)"
                @update:model-value="onToggle(row, $event)"
              />

              <AppSelect
                v-else-if="row.type === 'select'"
                :model-value="String(rowValue(row) || selectOptions(row)[0]?.value || '')"
                :options="selectOptions(row)"
                :label="rowLabel(row)"
                @update:model-value="onSelect(row, $event)"
              />

              <PathPicker
                v-else-if="row.type === 'path'"
                :model-value="String(rowValue(row) ?? '')"
                :label="rowLabel(row)"
                :placeholder="row.placeholder"
                :mode="row.mode"
                :filters="row.filters"
                @update:model-value="onText(row, $event)"
              />

              <SettingsTextField
                v-else-if="row.type === 'text'"
                :model-value="String(rowValue(row) ?? '')"
                :label="rowLabel(row)"
                :placeholder="row.placeholder"
                @update:model-value="onText(row, $event)"
              />

              <GhostButton v-else-if="row.type === 'button'" @click="onAction(row.action)">
                {{ row.buttonLabel }}
              </GhostButton>
            </PrefsRow>
          </div>
        </template>

        <button
          v-if="hasAdvanced"
          type="button"
          class="settings-advanced"
          @click="advancedOpen = !advancedOpen"
        >
          <span>{{ advancedOpen ? t("settings.advancedOpen") : t("settings.advanced") }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
