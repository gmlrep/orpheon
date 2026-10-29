<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { ACCENT_SWATCHES } from "../settings/accents";

const model = defineModel<number>({ default: 0 });

const { label } = defineProps<{
  label?: string;
}>();

const { t } = useI18n();
</script>

<template>
  <div class="accent-picker" role="listbox" :aria-label="label">
    <button
      v-for="(color, i) in ACCENT_SWATCHES"
      :key="color"
      type="button"
      class="accent-swatch"
      :class="{ 'is-active': model === i }"
      :style="{ backgroundColor: color }"
      :aria-label="t('settings.accentN', { n: i + 1 })"
      :aria-selected="model === i"
      @click="model = i"
    />
  </div>
</template>

<style scoped>
.accent-picker {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}
.accent-swatch {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 2px solid transparent;
  padding: 0;
}
.accent-swatch.is-active {
  border-color: var(--text);
}
</style>
