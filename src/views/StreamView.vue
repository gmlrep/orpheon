<script setup lang="ts">
import { computed, onMounted, shallowRef } from "vue";
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";
import { usePlaybackStore } from "../stores/playback";
import { useStreamsStore } from "../stores/streams";
import { resolveMediaSrc } from "../composables/useCoverSrc";
import AppIcon from "../components/AppIcon.vue";
import MediaThumb from "../components/MediaThumb.vue";
import type { SavedStream } from "../types";

defineOptions({ name: "StreamView" });

const { t } = useI18n();
const playback = usePlaybackStore();
const streams = useStreamsStore();
const { session, playing } = storeToRefs(playback);
const { saved, loading } = storeToRefs(streams);

const mrl = shallowRef("");
const error = shallowRef("");

const trimmed = computed(() => mrl.value.trim());
const currentIsInput = computed(() => !!trimmed.value && playback.isCurrentMrl(trimmed.value));

onMounted(() => {
  void streams.load();
});

async function openStream(target?: SavedStream) {
  error.value = "";
  const url = (target?.mrl ?? trimmed.value).trim();
  if (!url) return;
  try {
    if (playback.isCurrentMrl(url)) {
      await playback.playPause();
      return;
    }
    await playback.playMrl(url, {
      title: target?.title,
      artworkUrl: target?.artworkUrl,
    });
    mrl.value = url;
  } catch (e) {
    error.value = String(e);
  }
}

async function saveCurrent() {
  error.value = "";
  const url = trimmed.value;
  if (!url) return;
  try {
    const title =
      playback.isCurrentMrl(url) && session.value.title !== url ? session.value.title : undefined;
    const artworkUrl = playback.isCurrentMrl(url) ? session.value.artworkUrl : undefined;
    await streams.save(url, title, artworkUrl);
  } catch (e) {
    error.value = String(e);
  }
}

async function removeSaved(item: SavedStream) {
  error.value = "";
  try {
    await streams.remove(item.mrl);
  } catch (e) {
    error.value = String(e);
  }
}

function artSrc(item: SavedStream) {
  return resolveMediaSrc(item.artworkUrl);
}
</script>

<template>
  <div>
    <div class="title-row">
      <h1>{{ t("stream.title") }}</h1>
    </div>

    <div class="lib-bar">
      <label class="search stream-input">
        <input
          v-model="mrl"
          type="url"
          :placeholder="t('stream.placeholder')"
          @keydown.enter="openStream()"
        />
      </label>
      <button type="button" class="btn-primary" :disabled="!trimmed" @click="openStream()">
        <template v-if="currentIsInput">
          {{ playing ? t("player.pause") : t("player.play") }}
        </template>
        <template v-else>{{ t("stream.open") }}</template>
      </button>
      <button
        type="button"
        class="btn-ghost"
        :disabled="!trimmed || streams.isSaved(trimmed)"
        @click="saveCurrent"
      >
        {{ streams.isSaved(trimmed) ? t("stream.saved") : t("stream.save") }}
      </button>
    </div>

    <div v-if="session.active && session.path.includes('://')" class="now-stream">
      <div class="now-stream-art">
        <img
          v-if="resolveMediaSrc(session.artworkUrl)"
          :src="resolveMediaSrc(session.artworkUrl)!"
          alt=""
        />
        <MediaThumb v-else :title="session.title" :seed="session.path" :wide="false" />
      </div>
      <div class="now-stream-meta">
        <strong>{{ session.title }}</strong>
        <span>{{ session.subtitle || t("stream.live") }}</span>
      </div>
      <button
        type="button"
        class="play-mini"
        :aria-label="playing ? t('player.pause') : t('player.play')"
        @click="playback.playPause()"
      >
        <AppIcon :name="playing ? 'pause' : 'play'" :size="12" />
      </button>
    </div>

    <p v-if="error" class="empty stream-error">{{ error }}</p>

    <p class="section-label">{{ t("stream.savedList") }}</p>
    <p v-if="loading && !saved.length" class="empty">{{ t("common.loading") }}</p>
    <p v-else-if="!saved.length" class="empty">{{ t("stream.emptySaved") }}</p>
    <div v-else class="list-rows">
      <div v-for="item in saved" :key="item.mrl" class="stream-row">
        <button type="button" class="stream-main" @click="openStream(item)">
          <span class="stream-thumb">
            <img v-if="artSrc(item)" :src="artSrc(item)!" alt="" />
            <MediaThumb v-else :title="item.title" :seed="item.mrl" :wide="false" />
          </span>
          <span class="stream-meta">
            <strong>{{ item.title }}</strong>
            <span>{{ item.mrl }}</span>
          </span>
          <span
            v-if="playback.isCurrentMrl(item.mrl)"
            class="stream-state"
            :data-playing="playing || undefined"
          >
            {{ playing ? t("player.pause") : t("player.play") }}
          </span>
        </button>
        <button
          type="button"
          class="icon-btn"
          :aria-label="t('stream.remove')"
          @click="removeSaved(item)"
        >
          <AppIcon name="close" :size="14" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stream-input {
  max-width: 560px;
}
.stream-error {
  color: var(--negative);
}
.now-stream {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  margin-bottom: 20px;
  border-radius: var(--radius-card);
  background: var(--surface);
}
.now-stream-art {
  width: 44px;
  height: 44px;
  border-radius: 8px;
  overflow: hidden;
  flex: none;
}
.now-stream-art img,
.now-stream-art :deep(.art) {
  width: 100%;
  height: 100%;
  object-fit: cover;
  aspect-ratio: auto;
}
.now-stream-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.now-stream-meta strong {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.now-stream-meta span {
  font-size: 12px;
  color: var(--muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.stream-row {
  display: flex;
  align-items: center;
  gap: 4px;
}
.stream-main {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 12px;
  text-align: left;
  padding: 8px 10px;
  border-radius: 8px;
}
.stream-main:hover {
  background: var(--raised);
}
.stream-thumb {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  overflow: hidden;
  flex: none;
}
.stream-thumb img,
.stream-thumb :deep(.art) {
  width: 100%;
  height: 100%;
  object-fit: cover;
  aspect-ratio: auto;
}
.stream-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.stream-meta strong {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.stream-meta span {
  font-size: 11px;
  color: var(--faint);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.stream-state {
  flex: none;
  font-size: 11px;
  color: var(--muted);
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--raised);
}
.stream-state[data-playing] {
  color: var(--accent);
}
</style>
