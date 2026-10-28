<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { usePlaybackStore } from "../stores/playback";

const { t } = useI18n();
const playback = usePlaybackStore();
const mrl = ref("");
const error = ref("");

async function openStream() {
  error.value = "";
  try {
    await playback.playMrl(mrl.value.trim());
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <div>
    <div class="title-row">
      <h1>{{ t("stream.title") }}</h1>
    </div>
    <div class="lib-bar">
      <label class="search" style="max-width: 560px">
        <input v-model="mrl" type="url" :placeholder="t('stream.placeholder')" @keydown.enter="openStream" />
      </label>
      <button type="button" class="btn-primary" :disabled="!mrl.trim()" @click="openStream">
        {{ t("stream.open") }}
      </button>
    </div>
    <p v-if="error" class="empty" style="color: var(--negative)">{{ error }}</p>
  </div>
</template>
