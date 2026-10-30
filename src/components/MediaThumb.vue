<script setup lang="ts">
import { onMounted, shallowRef, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { DEFAULT_COVER_SRC } from "../assets/defaultCover";
import { resolveCoverSrc, resolveMediaSrc } from "../composables/useCoverSrc";
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

void title;
void seed;

const src = shallowRef(DEFAULT_COVER_SRC);
/** Session-local negative cache so remounts do not re-invoke while list data is stale. */
const noArt = shallowRef(false);

async function load() {
  // Prefer DB/cached art immediately — never block the grid on ffmpeg/VLC generate.
  // ":none" = backend already tried and found nothing (skip re-spawn).
  if (artworkPath === ":none" || noArt.value) {
    src.value = DEFAULT_COVER_SRC;
    return;
  }
  if (artworkPath) {
    src.value = resolveCoverSrc(artworkPath);
    return;
  }
  src.value = DEFAULT_COVER_SRC;
  if (!path) return;
  try {
    const art = await enqueueArt(() =>
      invoke<string | null>("resolve_artwork", {
        path,
        mediaId: mediaId ?? null,
      }),
    );
    if (art && resolveMediaSrc(art)) src.value = resolveCoverSrc(art);
    else noArt.value = true;
  } catch {
    noArt.value = true;
    src.value = DEFAULT_COVER_SRC;
  }
}

function onImgError() {
  if (src.value !== DEFAULT_COVER_SRC) {
    src.value = DEFAULT_COVER_SRC;
    noArt.value = true;
  }
}

onMounted(load);
watch(
  () => [path, artworkPath, mediaId] as const,
  () => {
    noArt.value = false;
    void load();
  },
);
</script>

<template>
  <div class="art" :class="{ square: !wide }" aria-hidden="true">
    <img class="photo" :src="src" alt="" @error="onImgError" />
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
