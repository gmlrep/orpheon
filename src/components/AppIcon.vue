<script setup lang="ts">
import { computed } from "vue";

const { name, variant = "default", size = 18 } = defineProps<{
  name: string;
  variant?: "default" | "active" | "twotone";
  size?: number;
}>();

const modules = import.meta.glob("../assets/icons/*.svg", {
  eager: true,
  query: "?raw",
  import: "default",
}) as Record<string, string>;

function resolveKey(base: string, v: string): string | null {
  const candidates =
    v === "active"
      ? [`${base}-active`, base]
      : v === "twotone"
        ? [`${base}-twotone`, base]
        : [base];
  for (const c of candidates) {
    const key = `../assets/icons/${c}.svg`;
    if (modules[key]) return key;
  }
  return null;
}

const svg = computed(() => {
  const key = resolveKey(name, variant);
  return key ? modules[key] : "";
});
</script>

<template>
  <!-- trusted local SVG assets only — never pass user/remote HTML here -->
  <span
    class="app-icon"
    :style="{ width: `${size}px`, height: `${size}px` }"
    v-html="svg"
    aria-hidden="true"
  />
</template>

<style scoped>
.app-icon {
  display: inline-grid;
  place-items: center;
  flex: none;
  line-height: 0;
  color: inherit;
}
.app-icon :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
}
</style>
