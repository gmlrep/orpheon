<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { usePlaybackStore } from "../stores/playback";
import { useUiStore } from "../stores/ui";

const { t } = useI18n();
const playback = usePlaybackStore();
const ui = useUiStore();

const enabled = ref(false);
const preamp = ref(0.5);
const bands = ref(Array.from({ length: 10 }, () => 0.5));
const labels = [
  "60 Hz",
  "170 Hz",
  "310 Hz",
  "600 Hz",
  "1 KHz",
  "3 KHz",
  "6 KHz",
  "12 KHz",
  "14 KHz",
  "16 KHz",
];

watch([enabled, preamp, bands], () => {
  void playback.applyEqualizer(enabled.value, preamp.value, [...bands.value]);
}, { deep: true });
</script>

<template>
  <div>
    <div class="title-row">
      <div>
        <h1>{{ t("eq.title") }}</h1>
      </div>
      <button type="button" class="btn-ghost" @click="ui.navigate('player')">Back</button>
    </div>

    <div class="settings-row">
      <div class="label">
        <strong>{{ t("eq.enable") }}</strong>
      </div>
      <button
        type="button"
        class="switch"
        role="switch"
        :aria-checked="enabled"
        @click="enabled = !enabled"
      >
        <i />
      </button>
    </div>

    <div class="settings-row">
      <div class="label">
        <strong>{{ t("eq.preamp") }}</strong>
      </div>
      <input v-model.number="preamp" type="range" min="0" max="1" step="0.01" style="width: 220px" />
    </div>

    <div style="display: grid; grid-template-columns: repeat(10, minmax(0, 1fr)); gap: 8px; margin-top: 24px">
      <label v-for="(label, i) in labels" :key="label" style="text-align: center">
        <input
          v-model.number="bands[i]"
          type="range"
          min="0"
          max="1"
          step="0.01"
          orient="vertical"
          style="writing-mode: vertical-lr; height: 140px; direction: rtl"
        />
        <span style="display: block; font-size: 11px; color: var(--faint); margin-top: 8px">{{
          label
        }}</span>
      </label>
    </div>
  </div>
</template>
