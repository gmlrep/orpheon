<script setup lang="ts">
import { computed, onMounted, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import type { DefaultMediaFolders } from "../types";
import { useLibraryStore } from "../stores/library";
import { useSettingsStore } from "../stores/settings";
import PathPicker from "./PathPicker.vue";

const emit = defineEmits<{
  done: [];
}>();

const { t } = useI18n();
const library = useLibraryStore();
const settings = useSettingsStore();

const defaults = shallowRef<DefaultMediaFolders | null>(null);
const useDefaultPath = ref(true);
const musicPath = ref("");
const videosPath = ref("");
const busy = ref(false);
const error = ref("");

const musicHint = computed(() => defaults.value?.music?.path ?? "");
const videosHint = computed(() => defaults.value?.videos?.path ?? "");
const musicValid = computed(() => Boolean(defaults.value?.music?.valid));
const videosValid = computed(() => Boolean(defaults.value?.videos?.valid));

const inputsLocked = computed(() => useDefaultPath.value);

function applyDefaultPaths() {
  musicPath.value = musicHint.value;
  videosPath.value = videosHint.value;
}

watch(useDefaultPath, (on) => {
  if (on) applyDefaultPaths();
});

onMounted(async () => {
  try {
    defaults.value = await invoke<DefaultMediaFolders>("default_media_folders");
  } catch {
    defaults.value = { music: null, videos: null };
  }
  if (useDefaultPath.value) applyDefaultPaths();
});

async function finish(skipFolders: boolean) {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    if (!skipFolders) {
      // Default paths: only scan folders that exist; custom: use whatever the user entered.
      const music = useDefaultPath.value
        ? musicValid.value
          ? musicHint.value
          : ""
        : musicPath.value.trim();
      const videos = useDefaultPath.value
        ? videosValid.value
          ? videosHint.value
          : ""
        : videosPath.value.trim();
      if (music) {
        try {
          await library.addFolder(music, "audio");
        } catch (e) {
          error.value = String(e);
          busy.value = false;
          return;
        }
      }
      if (videos) {
        try {
          await library.addFolder(videos, "video");
        } catch (e) {
          error.value = String(e);
          busy.value = false;
          return;
        }
      }
    }
    await settings.save({ onboarding_done: true });
    emit("done");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="onboard-scrim" role="presentation">
    <div class="onboard-card" role="dialog" aria-modal="true" :aria-label="t('onboarding.title')">
      <p class="step">{{ t("onboarding.step", { n: 1, total: 1 }) }}</p>
      <h1 class="title">{{ t("onboarding.title") }}</h1>
      <p class="lead">{{ t("onboarding.lead") }}</p>

      <label class="default-row">
        <input v-model="useDefaultPath" class="default-check" type="checkbox" />
        <span>{{ t("onboarding.defaultPath") }}</span>
      </label>

      <div class="fields">
        <label class="field">
          <span class="field-label">{{ t("onboarding.musicFolder") }}</span>
          <PathPicker
            v-model="musicPath"
            grow
            :label="t('onboarding.musicFolder')"
            :placeholder="musicHint || t('onboarding.musicPlaceholder')"
            :disabled="inputsLocked"
          />
          <span v-if="useDefaultPath && musicHint && !musicValid" class="hint warn">
            {{ t("onboarding.pathMissing") }}
          </span>
        </label>

        <label class="field">
          <span class="field-label">{{ t("onboarding.videosFolder") }}</span>
          <PathPicker
            v-model="videosPath"
            grow
            :label="t('onboarding.videosFolder')"
            :placeholder="videosHint || t('onboarding.videosPlaceholder')"
            :disabled="inputsLocked"
          />
          <span v-if="useDefaultPath && videosHint && !videosValid" class="hint warn">
            {{ t("onboarding.pathMissing") }}
          </span>
        </label>
      </div>

      <p v-if="error" class="error" role="alert">{{ error }}</p>

      <div class="actions">
        <button type="button" class="btn ghost" :disabled="busy" @click="finish(true)">
          {{ t("onboarding.skip") }}
        </button>
        <button type="button" class="btn primary" :disabled="busy" @click="finish(false)">
          {{ busy ? t("common.loading") : t("onboarding.continue") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.onboard-scrim {
  position: fixed;
  inset: 0;
  z-index: 80;
  display: grid;
  place-items: center;
  padding: 24px;
  background: var(--overlay-scrim);
}
.onboard-card {
  width: min(520px, 100%);
  padding: 28px 28px 22px;
  border-radius: 16px;
  border: 1px solid var(--border);
  background: var(--surface);
  box-shadow: var(--shadow-float);
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.step {
  margin: 0;
  font-size: 12px;
  color: var(--faint);
  letter-spacing: 0.04em;
  text-transform: uppercase;
}
.title {
  margin: 0;
  font-size: var(--fs-title);
  font-weight: 600;
  color: var(--text);
  line-height: 1.2;
}
.lead {
  margin: 0;
  font-size: 14px;
  color: var(--muted);
  line-height: 1.45;
}
.default-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 4px;
  font-size: 13px;
  color: var(--text);
  cursor: pointer;
  user-select: none;
}
.default-check {
  width: 16px;
  height: 16px;
  accent-color: var(--accent);
  cursor: pointer;
}
.fields {
  display: flex;
  flex-direction: column;
  gap: 14px;
  margin-top: 4px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.field-label {
  font-size: 12px;
  color: var(--muted);
}
.hint {
  font-size: 12px;
  color: var(--faint);
}
.hint.warn {
  color: var(--warning);
}
.error {
  margin: 0;
  font-size: 12px;
  color: var(--negative);
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
.btn {
  height: 36px;
  padding: 0 16px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 500;
  border: 1px solid var(--border);
  cursor: pointer;
}
.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.btn.ghost {
  background: transparent;
  color: var(--muted);
}
.btn.ghost:hover:not(:disabled) {
  color: var(--text);
  background: var(--raised);
}
.btn.primary {
  background: var(--primary);
  color: var(--primary-ink);
  border-color: transparent;
}
.btn.primary:hover:not(:disabled) {
  filter: brightness(0.96);
}
</style>
