<script setup lang="ts">
const { label } = defineProps<{
  label?: string;
}>();

const emit = defineEmits<{
  click: [];
}>();
</script>

<template>
  <button type="button" class="ghost-btn" :aria-label="label" @click="emit('click')">
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
.ghost-btn:hover {
  color: var(--text);
}
</style>
