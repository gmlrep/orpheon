<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, watch } from "vue";

const model = defineModel<number>({ required: true });
const emit = defineEmits<{
  scrubStart: [];
  scrubEnd: [];
}>();

const track = useTemplateRef<HTMLElement>("track");
const dragging = shallowRef(false);
const local = shallowRef(model.value);

watch(model, (v) => {
  if (!dragging.value) local.value = v;
});

const display = computed(() => (dragging.value ? local.value : model.value));

function posFromEvent(e: PointerEvent) {
  const el = track.value;
  if (!el) return 0;
  const r = el.getBoundingClientRect();
  if (r.width <= 0) return 0;
  return Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
}

function onPointerDown(e: PointerEvent) {
  const el = track.value;
  if (!el) return;
  dragging.value = true;
  emit("scrubStart");
  el.setPointerCapture(e.pointerId);
  const p = posFromEvent(e);
  local.value = p;
  model.value = p;
}

function onPointerMove(e: PointerEvent) {
  if (!dragging.value) return;
  const p = posFromEvent(e);
  local.value = p;
  model.value = p;
}

function onPointerUp(e: PointerEvent) {
  if (!dragging.value) return;
  dragging.value = false;
  emit("scrubEnd");
  try {
    track.value?.releasePointerCapture(e.pointerId);
  } catch {
    /* ignore */
  }
}

function onKey(e: KeyboardEvent) {
  let next = display.value;
  if (e.key === "ArrowLeft" || e.key === "ArrowDown") next = Math.max(0, next - 0.01);
  else if (e.key === "ArrowRight" || e.key === "ArrowUp") next = Math.min(1, next + 0.01);
  else if (e.key === "Home") next = 0;
  else if (e.key === "End") next = 1;
  else return;
  e.preventDefault();
  local.value = next;
  model.value = next;
}
</script>

<template>
  <div
    ref="track"
    class="seekbar"
    role="slider"
    tabindex="0"
    :aria-valuemin="0"
    :aria-valuemax="100"
    :aria-valuenow="Math.round(display * 100)"
    :aria-label="'Seek'"
    :style="{ '--p': display }"
    :data-dragging="dragging || undefined"
    @pointerdown.prevent="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
    @keydown="onKey"
  >
    <span class="seekbar-fill" aria-hidden="true" />
    <span class="seekbar-head" aria-hidden="true" />
  </div>
</template>

<style scoped>
.seekbar {
  position: relative;
  flex: 1 1 auto;
  min-width: 80px;
  height: 28px;
  border-radius: 6px;
  background: var(--seek-track);
  cursor: pointer;
  touch-action: none;
  user-select: none;
}
.seekbar-fill {
  position: absolute;
  left: 0;
  top: 2px;
  height: 24px;
  width: calc(var(--p, 0) * 100%);
  max-width: 100%;
  border-radius: 4px;
  background: var(--seek-fill);
  pointer-events: none;
}
.seekbar-head {
  position: absolute;
  top: 2px;
  left: calc(var(--p, 0) * 100%);
  width: 8px;
  height: 24px;
  margin-left: -4px;
  border-radius: 4px;
  background: var(--accent);
  pointer-events: none;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35);
}
.seekbar:hover .seekbar-head,
.seekbar[data-dragging] .seekbar-head {
  filter: brightness(1.08);
}
</style>
