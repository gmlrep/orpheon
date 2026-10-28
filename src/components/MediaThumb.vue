<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { toneFor } from "../utils/format";

const props = withDefaults(
  defineProps<{
    title: string;
    seed?: string | number;
    path?: string;
    mediaId?: number | null;
    artworkPath?: string | null;
    wide?: boolean;
    progress?: number;
  }>(),
  { wide: true, progress: undefined, mediaId: null, artworkPath: null },
);

const src = ref<string | null>(null);
const failed = ref(false);
const palette = computed(() => toneFor(props.seed ?? props.title));

/** Limit concurrent VLC/ffmpeg thumbnail jobs so opening a list doesn't spawn dozens. */
const artQueue: Array<() => void> = [];
let artRunning = 0;
const ART_MAX = 2;

function enqueueArt<T>(job: () => Promise<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    const run = () => {
      artRunning++;
      job()
        .then(resolve, reject)
        .finally(() => {
          artRunning--;
          const next = artQueue.shift();
          if (next) next();
        });
    };
    if (artRunning < ART_MAX) run();
    else artQueue.push(run);
  });
}

function toSrc(filePath: string): string {
  return convertFileSrc(filePath.replace(/\\/g, "/"));
}

async function load() {
  failed.value = false;
  src.value = null;
  if (props.path) {
    try {
      const art = await enqueueArt(() =>
        invoke<string | null>("resolve_artwork", {
          path: props.path,
          mediaId: props.mediaId ?? null,
        }),
      );
      if (art) {
        src.value = toSrc(art);
        return;
      }
    } catch {
      /* fall through */
    }
  }
  if (props.artworkPath) {
    src.value = toSrc(props.artworkPath);
  }
}

onMounted(load);
watch(() => [props.path, props.artworkPath, props.mediaId], load);
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
  background: #151515;
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
  background: #2a2a2a;
}
.progress > i {
  display: block;
  height: 100%;
  width: calc(var(--p, 0) * 100%);
  background: var(--accent);
}
</style>
