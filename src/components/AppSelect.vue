<script setup lang="ts" generic="T extends string | number | boolean">
import { computed, nextTick, onBeforeUnmount, onMounted, shallowRef, useTemplateRef } from "vue";
import type { AppSelectOption } from "./appSelectTypes";

const model = defineModel<T>({ required: true });

const {
  options,
  label,
  placeholder = "—",
} = defineProps<{
  options: AppSelectOption<T>[];
  label?: string;
  placeholder?: string;
}>();

const open = shallowRef(false);
const rootEl = useTemplateRef<HTMLElement>("root");

const selectedLabel = computed(() => {
  const hit = options.find((o) => o.value === model.value);
  return hit?.label ?? placeholder;
});

function toggle() {
  open.value = !open.value;
}

function choose(opt: AppSelectOption<T>) {
  model.value = opt.value;
  open.value = false;
}

function onDocPointer(e: PointerEvent) {
  if (!open.value) return;
  const el = rootEl.value;
  if (el && !el.contains(e.target as Node)) open.value = false;
}

function onKey(e: KeyboardEvent) {
  if (!open.value) return;
  if (e.key === "Escape") {
    e.preventDefault();
    open.value = false;
  }
}

onMounted(async () => {
  await nextTick();
  document.addEventListener("pointerdown", onDocPointer, true);
  document.addEventListener("keydown", onKey, true);
});

onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", onDocPointer, true);
  document.removeEventListener("keydown", onKey, true);
});
</script>

<template>
  <div ref="root" class="app-select" :data-open="open || undefined">
    <button
      type="button"
      class="app-select-trigger"
      :aria-label="label"
      :aria-expanded="open"
      aria-haspopup="listbox"
      @click="toggle"
    >
      <span class="app-select-value">{{ selectedLabel }}</span>
      <span class="app-select-chevron" aria-hidden="true">▾</span>
    </button>
    <ul v-if="open" class="app-select-menu" role="listbox" :aria-label="label">
      <li v-for="opt in options" :key="String(opt.value)" role="none">
        <button
          type="button"
          class="app-select-option"
          role="option"
          :aria-selected="model === opt.value"
          @click="choose(opt)"
        >
          {{ opt.label }}
        </button>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.app-select {
  position: relative;
  flex: none;
}
.app-select-trigger {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 120px;
  max-width: 240px;
  height: 32px;
  padding: 0 10px 0 12px;
  border-radius: 8px;
  background: var(--raised);
  color: var(--text);
  font-size: 12px;
  border: 1px solid var(--border);
}
.app-select-value {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: left;
}
.app-select-chevron {
  color: var(--faint);
  font-size: 10px;
  line-height: 1;
}
.app-select-menu {
  position: absolute;
  right: 0;
  top: calc(100% + 4px);
  z-index: 20;
  margin: 0;
  padding: 4px;
  list-style: none;
  min-width: 100%;
  max-height: 240px;
  overflow: auto;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
}
.app-select-option {
  display: block;
  width: 100%;
  text-align: left;
  padding: 8px 10px;
  border-radius: 6px;
  color: var(--muted);
  font-size: 12px;
}
.app-select-option:hover {
  background: var(--raised);
  color: var(--text);
}
.app-select-option[aria-selected="true"] {
  background: var(--active);
  color: var(--text);
}
</style>
