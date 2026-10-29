<script setup lang="ts">
const checked = defineModel<boolean>({ default: false });

const { label } = defineProps<{
  label?: string;
}>();
</script>

<template>
  <button
    type="button"
    class="app-switch"
    role="switch"
    :aria-checked="checked"
    :aria-label="label"
    @click="checked = !checked"
  >
    <span class="app-switch-knob" aria-hidden="true" />
  </button>
</template>

<style scoped>
.app-switch {
  width: 52px;
  height: 28px;
  border-radius: 6px;
  background: var(--switch-track);
  position: relative;
  flex: none;
  border: 0;
  padding: 0;
}
.app-switch[aria-checked="true"] {
  background: var(--accent);
}
.app-switch-knob {
  position: absolute;
  top: 4px;
  left: 4px;
  width: 20px;
  height: 20px;
  border-radius: 4px;
  background: var(--switch-knob);
  transition:
    transform var(--t-hover) var(--ease),
    background var(--t-hover) var(--ease);
  pointer-events: none;
}
.app-switch[aria-checked="true"] .app-switch-knob {
  transform: translateX(24px);
  background: var(--switch-knob-on);
}
</style>
