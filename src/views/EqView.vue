<script setup lang="ts">
import { computed, onMounted, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { usePlaybackStore } from "../stores/playback";
import { useSettingsStore } from "../stores/settings";
import { useUiStore } from "../stores/ui";
import AppSwitch from "../components/AppSwitch.vue";
import { EQ_APPLY_DEBOUNCE_MS, EQ_BAND_COUNT, EQ_BAND_KEYS, EQ_DEFAULT_GAIN } from "../config/app";

defineOptions({ name: "EqView" });

const { t } = useI18n();
const playback = usePlaybackStore();
const settings = useSettingsStore();
const ui = useUiStore();

const enabled = shallowRef(false);
const preamp = shallowRef(EQ_DEFAULT_GAIN);
/** Deep ref: band sliders mutate indices in place. */
const bands = ref(Array.from({ length: EQ_BAND_COUNT }, () => EQ_DEFAULT_GAIN));
const bandKeys = EQ_BAND_KEYS;
const labels = computed(() => bandKeys.map((k) => t(`eq.bands.${k}`)));

onMounted(() => {
  enabled.value = settings.prefBool("equalizer_enabled", false);
});

let applyTimer = 0;
let syncing = false;
watch(
  [enabled, preamp, bands],
  () => {
    window.clearTimeout(applyTimer);
    applyTimer = window.setTimeout(() => {
      void playback.applyEqualizer(enabled.value, preamp.value, [...bands.value]);
      if (!syncing && settings.prefBool("equalizer_enabled", false) !== enabled.value) {
        syncing = true;
        void settings.setPref("equalizer_enabled", enabled.value).finally(() => {
          syncing = false;
        });
      }
    }, EQ_APPLY_DEBOUNCE_MS);
  },
  { deep: true },
);
</script>

<template>
  <div>
    <div class="title-row">
      <div>
        <h1>{{ t("eq.title") }}</h1>
      </div>
      <button type="button" class="btn-ghost" @click="ui.navigate('player')">
        {{ t("common.back") }}
      </button>
    </div>

    <div class="settings-row">
      <div class="label">
        <strong>{{ t("eq.enable") }}</strong>
      </div>
      <AppSwitch v-model="enabled" :label="t('eq.enable')" />
    </div>

    <div class="settings-row">
      <div class="label">
        <strong>{{ t("eq.preamp") }}</strong>
      </div>
      <input v-model.number="preamp" type="range" min="0" max="1" step="0.01" class="preamp" />
    </div>

    <div class="bands">
      <label v-for="(label, i) in labels" :key="bandKeys[i]" class="band">
        <input
          v-model.number="bands[i]"
          type="range"
          min="0"
          max="1"
          step="0.01"
          orient="vertical"
          class="band-slider"
        />
        <span class="band-label">{{ label }}</span>
      </label>
    </div>
  </div>
</template>

<style scoped>
.preamp {
  width: 220px;
}
.bands {
  display: grid;
  grid-template-columns: repeat(10, minmax(0, 1fr));
  gap: 8px;
  margin-top: 24px;
}
.band {
  text-align: center;
}
.band-slider {
  writing-mode: vertical-lr;
  height: 140px;
  direction: rtl;
}
.band-label {
  display: block;
  font-size: 11px;
  color: var(--faint);
  margin-top: 8px;
}
</style>
