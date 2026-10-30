<script setup lang="ts">
const { label, disabled = false } = defineProps<{
  label?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  click: [];
}>();
</script>

<template>
  <button
    type="button"
    class="ghost-btn"
    :aria-label="label"
    :disabled="disabled"
    @click="emit('click')"
  >
    <slot />
  </button>
</template>

<style scoped>
.ghost-btn {
  height: 32px;
  padding: 0 12px;
  border-radius: 8px;
  background: var(--raised);
  color: var(--muted);
  font-size: 12px;
  border: 1px solid var(--border);
}
.ghost-btn:hover:not(:disabled) {
  color: var(--text);
}
.ghost-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
