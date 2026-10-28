<script setup lang="ts">
import { computed, ref } from "vue";
import AppIcon from "./AppIcon.vue";

const props = defineProps<{
  value: number; // 0..200 (libVLC softvol), UI maps 0..1 of max 125 by default → use 0..100 as percent of 100
  muted?: boolean;
}>();
const emit = defineEmits<{
  change: [number];
  toggleMute: [];
}>();

/** Display level 0..1 from softvol 0..200 (100 = unity). Cap visual at 125%. */
const level = computed(() => {
  if (props.muted) return 0;
  return Math.min(1, Math.max(0, props.value / 125));
});

const track = ref<HTMLElement | null>(null);
const dragging = ref(false);

function levelFromEvent(e: PointerEvent) {
  const el = track.value;
  if (!el) return 0;
  const r = el.getBoundingClientRect();
  if (r.width <= 0) return 0;
  return Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
}

function emitFromLevel(lv: number) {
  // Map 0..1 visual → 0..125 softvol units
  emit("change", Math.round(lv * 125));
}

function onPointerDown(e: PointerEvent) {
  const el = track.value;
  if (!el) return;
  dragging.value = true;
  el.setPointerCapture(e.pointerId);
  emitFromLevel(levelFromEvent(e));
}
function onPointerMove(e: PointerEvent) {
  if (!dragging.value) return;
  emitFromLevel(levelFromEvent(e));
}
function onPointerUp(e: PointerEvent) {
  if (!dragging.value) return;
  dragging.value = false;
  try {
    track.value?.releasePointerCapture(e.pointerId);
  } catch {
    /* ignore */
  }
}
</script>

<template>
  <div class="vol">
    <button type="button" class="icon-btn" aria-label="Mute" @click="emit('toggleMute')">
      <AppIcon name="volume" :size="18" />
    </button>
    <div
      ref="track"
      class="wedge"
      role="slider"
      tabindex="0"
      :aria-valuenow="Math.round(level * 100)"
      :aria-valuemin="0"
      :aria-valuemax="100"
      :aria-label="'Volume'"
      :style="{ '--v': level }"
      @pointerdown.prevent="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
    >
      <img class="wedge-bg" src="../assets/icons/volume-wedge-bg.svg" alt="" draggable="false" />
      <span class="wedge-clip">
        <img class="wedge-fill" src="../assets/icons/volume-wedge-fill.svg" alt="" draggable="false" />
      </span>
    </div>
  </div>
</template>

<style scoped>
.vol {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: none;
  height: 28px;
}
.wedge {
  position: relative;
  width: 88px;
  height: 28px;
  cursor: pointer;
  touch-action: none;
  user-select: none;
  flex: none;
}
.wedge-bg,
.wedge-fill {
  position: absolute;
  inset: 0;
  width: 88px;
  height: 28px;
  display: block;
  pointer-events: none;
}
.wedge-clip {
  position: absolute;
  inset: 0 auto 0 0;
  width: calc(var(--v, 0) * 88px);
  overflow: hidden;
  pointer-events: none;
}
</style>
