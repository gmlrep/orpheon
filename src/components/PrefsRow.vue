<script setup lang="ts">
const { label, hint } = defineProps<{
  label: string;
  hint?: string;
}>();
</script>

<template>
  <div class="settings-row">
    <div class="label">
      <strong class="settings-row-title">{{ label }}</strong>
      <span v-if="hint" class="settings-row-hint">{{ hint }}</span>
    </div>
    <div class="settings-ctrl">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 0;
  border-bottom: 1px solid var(--rule);
}
.label {
  min-width: 0;
}
.settings-row-title {
  display: block;
  font-weight: 500;
}
.settings-row-hint {
  display: block;
  margin-top: 2px;
  font-size: 12px;
  color: var(--faint);
  max-width: 42ch;
  line-height: 1.35;
}
.settings-ctrl {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: none;
}
</style>
