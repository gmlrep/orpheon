<script setup lang="ts">
export interface SegmentOption {
  value: string;
  label: string;
}

const model = defineModel<string>({ required: true });

const { options, label } = defineProps<{
  options: SegmentOption[];
  label?: string;
}>();
</script>

<template>
  <div class="seg" role="group" :aria-label="label">
    <button
      v-for="opt in options"
      :key="opt.value"
      type="button"
      :aria-pressed="model === opt.value"
      @click="model = opt.value"
    >
      {{ opt.label }}
    </button>
  </div>
</template>

<style scoped>
.seg {
  display: inline-flex;
  background: var(--raised);
  border-radius: 8px;
  padding: 0;
  gap: 0;
}
.seg button {
  height: 28px;
  padding: 0 12px;
  border-radius: 0;
  color: var(--muted);
  font-size: 12px;
  min-width: 72px;
}
.seg button:first-child {
  border-radius: 8px 0 0 8px;
}
.seg button:last-child {
  border-radius: 0 8px 8px 0;
}
.seg button[aria-pressed="true"] {
  background: var(--seg-active-bg);
  color: var(--seg-active-ink);
}
</style>
