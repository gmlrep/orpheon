<script setup lang="ts">
import { computed, onMounted, shallowRef, watch } from "vue";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { toneFor } from "../utils/format";
import { enqueueArt } from "../utils/artQueue";

const {
  title,
  seed,
  path,
  mediaId = null,
  artworkPath = null,
  wide = true,
  progress,
} = defineProps<{
  title: string;
  seed?: string | number;
  path?: string;
  mediaId?: number | null;
  artworkPath?: string | null;
  wide?: boolean;
  progress?: number;
}>();

const src = shallowRef<string | null>(null);
const failed = shallowRef(false);
const palette = computed(() => toneFor(seed ?? title));

function toSrc(filePath: string): string {
  return convertFileSrc(filePath.replace(/\\/g, "/"));
}

async function load() {
  failed.value = false;
  // Prefer DB/cached art immediately — never block the grid on ffmpeg/VLC generate.
  if (artworkPath) {
    src.value = toSrc(artworkPath);
    return;
  }
  src.value = null;
  if (!path) return;
  try {
    const art = await enqueueArt(() =>
      invoke<string | null>("resolve_artwork", {
        path,
        mediaId: mediaId ?? null,
      }),
    );
    if (art) src.value = toSrc(art);
  } catch {
    /* keep placeholder */
  }
}

onMounted(load);
watch(() => [path, artworkPath, mediaId], load);
</script>

<template>
  <div class="art" :class="{ square: !wide }" aria-hidden="true">
    <img v-if="src && !failed" class="photo" :src="src" alt="" @error="failed = true" />
    <div v-else class="placeholder" :style="{ background: palette[0] }">
      <span class="glow" :style="{ background: palette[1] }" />
    </div>
    <div v-if="progress != null" class="progress" :style="{ '--p': progress }">
      <i />
    </div>
  </div>
</template>

<style scoped>
.art {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  overflow: hidden;
  background: var(--thumb-bg);
}
.art.square {
  aspect-ratio: 1;
}
.photo {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.placeholder {
  width: 100%;
  height: 100%;
  position: relative;
}
.glow {
  position: absolute;
  inset: 20% 25%;
  border-radius: 50%;
  opacity: 0.25;
  filter: blur(12px);
}
.progress {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 3px;
  background: var(--switch-track);
}
.progress > i {
  display: block;
  height: 100%;
  width: calc(var(--p, 0) * 100%);
  background: var(--accent);
}
</style>
