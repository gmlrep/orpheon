<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { storeToRefs } from "pinia";
import { useLibraryStore } from "../stores/library";
import { usePlaybackStore } from "../stores/playback";
import { useUiStore } from "../stores/ui";
import MediaThumb from "../components/MediaThumb.vue";
import { formatDuration } from "../utils/format";
import { LISTEN_AGAIN_LIMIT } from "../config/app";

defineOptions({ name: "HomeView" });

const { t } = useI18n();
const library = useLibraryStore();
const playback = usePlaybackStore();
const ui = useUiStore();
const { resume, recent, folders, loading } = storeToRefs(library);

const listenAgain = computed(() =>
  recent.value.filter((m) => m.media_type === "audio").slice(0, LISTEN_AGAIN_LIMIT),
);

const continueItems = computed(() => resume.value.slice(0, 3));

onMounted(() => {
  void library.loadHome();
});
</script>

<template>
  <div>
    <div class="title-row">
      <div>
        <h1>{{ t("home.title") }}</h1>
        <p class="sub">{{ t("app.tagline") }}</p>
      </div>
    </div>

    <p v-if="!loading && !resume.length && !listenAgain.length && !folders.length" class="empty">
      {{ t("home.empty") }}
    </p>

    <template v-if="continueItems.length">
      <div class="section-label">{{ t("home.continue") }}</div>
      <div class="resume-grid">
        <button
          v-for="item in continueItems"
          :key="item.path"
          type="button"
          class="resume-card"
          @click="playback.playResumeItem(item)"
        >
          <div class="art">
            <MediaThumb
              :title="item.title"
              :seed="item.path"
              :path="item.path"
              :media-id="item.id"
              :artwork-path="item.artworkPath"
              :progress="item.progress"
            />
          </div>
          <div class="body-text">
            <strong>{{ item.title }}</strong>
            <span>{{ formatDuration(item.durationMs) }}</span>
          </div>
        </button>
      </div>
    </template>

    <template v-if="listenAgain.length">
      <div class="section-label">{{ t("home.listen") }}</div>
      <div class="list-rows">
        <button
          v-for="item in listenAgain"
          :key="item.id"
          type="button"
          @click="playback.playMediaItem(item)"
        >
          <span class="thumb">
            <MediaThumb
              :title="item.title"
              :seed="item.path"
              :path="item.path"
              :media-id="item.id"
              :artwork-path="item.artwork_path"
              :wide="false"
            />
          </span>
          <span class="meta">
            <strong>{{ item.title }}</strong>
            <span>{{ formatDuration(item.duration_ms) }}</span>
          </span>
        </button>
      </div>
    </template>

    <template v-if="folders.length">
      <div class="section-label folders-label">{{ t("home.folders") }}</div>
      <div class="list-rows">
        <button
          v-for="folder in folders"
          :key="folder.path"
          type="button"
          @click="ui.navigate(folder.kind === 'audio' || folder.kind === 'music' ? 'music' : 'videos')"
        >
          <span class="meta">
            <strong>{{ folder.name }}</strong>
            <span>{{ folder.count }} items</span>
          </span>
        </button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.folders-label {
  margin-top: 24px;
}
</style>
