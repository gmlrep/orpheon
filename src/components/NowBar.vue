<script setup lang="ts">
import { computed } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { storeToRefs } from "pinia";
import { usePlaybackStore } from "../stores/playback";
import AppIcon from "./AppIcon.vue";
import MediaThumb from "./MediaThumb.vue";

const playback = usePlaybackStore();
const { session, playing, showNowBar, coverPath } = storeToRefs(playback);

const coverSrc = computed(() =>
  coverPath.value ? convertFileSrc(coverPath.value.replace(/\\/g, "/")) : null,
);
</script>

<template>
  <div v-if="showNowBar" class="now-bar" role="group" aria-label="Now playing">
    <button type="button" class="now-main" aria-label="Open now playing" @click="playback.openPlayer()">
      <span class="thumb">
        <img v-if="coverSrc" class="thumb-img" :src="coverSrc" alt="" />
        <MediaThumb
          v-else
          :title="session.title"
          :seed="session.path"
          :path="session.path"
          :media-id="session.mediaId"
          :wide="false"
        />
        <span class="prog" :style="{ '--p': session.position }"><i /></span>
      </span>
      <span class="meta">
        <strong>{{ session.title }}</strong>
        <span>{{ session.subtitle }}</span>
      </span>
    </button>
    <div class="now-transport" aria-label="Playback">
      <button type="button" class="icon-btn" aria-label="Previous track" @click="playback.previousTrack()">
        <AppIcon name="previous" :size="14" />
      </button>
      <button
        type="button"
        class="play-mini"
        :aria-label="playing ? 'Pause' : 'Play'"
        @click="playback.playPause()"
      >
        <AppIcon :name="playing ? 'pause' : 'play'" :size="12" />
      </button>
      <button type="button" class="icon-btn" aria-label="Next track" @click="playback.nextTrack()">
        <AppIcon name="next" :size="14" />
      </button>
    </div>
    <button
      type="button"
      class="icon-btn now-close"
      :aria-label="$t('common.stop')"
      @click="playback.stop()"
    >
      <AppIcon name="close" :size="14" />
    </button>
  </div>
</template>

<style scoped>
.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.thumb {
  position: relative;
}
.thumb :deep(.art) {
  width: 28px;
  height: 28px;
  aspect-ratio: auto;
}
</style>
