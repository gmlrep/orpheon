<script setup lang="ts">
import { useI18n } from "vue-i18n";
import AppIcon from "./AppIcon.vue";

export type PopupMenuRow =
  | { type: "sep" }
  | {
      type: "item";
      id: string;
      labelKey: string;
      /** When set, shown instead of `t(labelKey)`. */
      label?: string;
      icon?: string;
      shortcut?: string;
      disabled?: boolean;
    };

defineProps<{
  rows: PopupMenuRow[];
  playing?: boolean;
}>();

const emit = defineEmits<{
  pick: [id: string];
}>();

const { t } = useI18n();

function labelFor(row: Extract<PopupMenuRow, { type: "item" }>, playing?: boolean) {
  if (row.id === "play-pause") {
    return playing ? t("player.pause") : t("player.play");
  }
  if (row.label) return row.label;
  return t(row.labelKey);
}

function iconFor(row: Extract<PopupMenuRow, { type: "item" }>, playing?: boolean) {
  if (row.id === "play-pause") return playing ? "pause" : "play";
  return row.icon;
}
</script>

<template>
  <div class="popup-menu" role="menu">
    <template v-for="(row, i) in rows" :key="row.type === 'sep' ? `sep-${i}` : row.id">
      <div v-if="row.type === 'sep'" class="popup-sep" role="separator" />
      <button
        v-else
        type="button"
        class="popup-item"
        role="menuitem"
        :disabled="row.disabled"
        @click="emit('pick', row.id)"
      >
        <span class="popup-icon" aria-hidden="true">
          <AppIcon v-if="iconFor(row, playing)" :name="iconFor(row, playing)!" :size="20" />
        </span>
        <span class="popup-label">{{ labelFor(row, playing) }}</span>
        <span v-if="row.shortcut" class="popup-shortcut">{{ row.shortcut }}</span>
      </button>
    </template>
  </div>
</template>

<style scoped>
.popup-menu {
  min-width: 265px;
  max-width: 375px;
  padding: 4px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: color-mix(in srgb, var(--raised) 95%, transparent);
  box-shadow: var(--shadow-float);
  backdrop-filter: blur(12px);
}
.popup-sep {
  height: 1px;
  margin: 4px 0;
  background: var(--border);
}
.popup-item {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  width: 100%;
  height: 48px;
  padding: 0 14px 0 14px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--muted);
  text-align: left;
  cursor: pointer;
}
.popup-item:hover:not(:disabled),
.popup-item:focus-visible:not(:disabled) {
  background: var(--active);
  color: var(--text);
  outline: none;
}
.popup-item:disabled {
  opacity: 0.4;
  cursor: default;
}
.popup-icon {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  color: inherit;
}
.popup-label {
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.popup-shortcut {
  font-size: 14px;
  color: var(--faint);
  white-space: nowrap;
  padding-left: 12px;
}
</style>
