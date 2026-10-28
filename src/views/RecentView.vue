<script setup lang="ts">
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { storeToRefs } from "pinia";
import { useLibraryStore } from "../stores/library";
import { usePlaybackStore } from "../stores/playback";
import MediaThumb from "../components/MediaThumb.vue";
import { formatDuration } from "../utils/format";

const { t } = useI18n();
const library = useLibraryStore();
const playback = usePlaybackStore();
const { recent } = storeToRefs(library);

onMounted(() => {
  void library.loadRecent();
});

function play(item: (typeof recent.value)[number]) {
  const isVideo = item.media_type !== "audio";
  const q = playback.mediaToQueue(recent.value, isVideo);
  const idx = recent.value.findIndex((m) => m.id === item.id);
  void playback.playPath(item.path, {
    title: item.title,
    mediaId: item.id,
    isVideo,
    artworkPath: item.artwork_path,
    queue: q,
    queueIndex: Math.max(0, idx),
  });
}
</script>

<template>
  <div>
    <div class="title-row">
      <h1>{{ t("nav.recent") }}</h1>
    </div>
    <p v-if="!recent.length" class="empty">No recent media</p>
    <div v-else class="list-rows">
      <button v-for="item in recent" :key="item.id" type="button" @click="play(item)">
        <span class="thumb" :class="{ square: item.media_type === 'audio' }">
          <MediaThumb
            :title="item.title"
            :seed="item.path"
            :path="item.path"
            :media-id="item.id"
            :artwork-path="item.artwork_path"
            :wide="item.media_type !== 'audio'"
          />
        </span>
        <span class="meta">
          <strong>{{ item.title }}</strong>
          <span>{{ formatDuration(item.duration_ms) }}</span>
        </span>
      </button>
    </div>
  </div>
</template>
