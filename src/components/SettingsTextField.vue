<script setup lang="ts">
const model = defineModel<string>({ default: "" });

const { label, placeholder } = defineProps<{
  label?: string;
  placeholder?: string;
}>();
</script>

<template>
  <input
    class="settings-field"
    type="text"
    :value="model"
    :placeholder="placeholder"
    :aria-label="label"
    @input="model = ($event.target as HTMLInputElement).value"
    @change="model = ($event.target as HTMLInputElement).value"
  />
</template>

<style scoped>
.settings-field {
  min-width: 140px;
  max-width: 240px;
  height: 32px;
  padding: 0 10px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--raised);
  color: var(--text);
}
</style>
