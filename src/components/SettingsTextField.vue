<script setup lang="ts">
const model = defineModel<string>({ default: "" });

const {
  label,
  placeholder,
  disabled = false,
  grow = false,
} = defineProps<{
  label?: string;
  placeholder?: string;
  disabled?: boolean;
  grow?: boolean;
}>();
</script>

<template>
  <input
    class="settings-field"
    :class="{ grow }"
    type="text"
    :value="model"
    :placeholder="placeholder"
    :aria-label="label"
    :disabled="disabled"
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
.settings-field.grow {
  flex: 1;
  min-width: 0;
  max-width: none;
  width: 100%;
}
.settings-field:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
</style>
