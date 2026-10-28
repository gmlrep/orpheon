<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { storeToRefs } from "pinia";
import { useLibraryStore } from "../stores/library";
import { usePlaybackStore } from "../stores/playback";
import { useUiStore } from "../stores/ui";
import AppIcon from "../components/AppIcon.vue";
import MediaThumb from "../components/MediaThumb.vue";
import { formatDuration } from "../utils/format";
import type { MediaItem } from "../types";

const props = defineProps<{ kind: "video" | "audio" }>();

const { t } = useI18n();
const library = useLibraryStore();
const playback = usePlaybackStore();
const ui = useUiStore();
const { videos, music } = storeToRefs(library);
const { listQuery } = storeToRefs(ui);
const sortOpen = ref(false);

const items = computed(() => (props.kind === "video" ? videos.value : music.value));

const sorted = computed(() => {
  const copy = [...items.value];
  if (props.kind === "video") {
    const key = ui.videoSort;
    if (key === "title") copy.sort((a, b) => a.title.localeCompare(b.title));
    else if (key === "duration")
      copy.sort((a, b) => (a.duration_ms ?? 0) - (b.duration_ms ?? 0));
    else copy.sort((a, b) => b.id - a.id);
  } else {
    const key = ui.musicSort;
    if (key === "duration") copy.sort((a, b) => (a.duration_ms ?? 0) - (b.duration_ms ?? 0));
    else copy.sort((a, b) => a.title.localeCompare(b.title));
  }
  return copy;
});

async function reload() {
  if (props.kind === "video") await library.loadVideos(ui.listQuery);
  else await library.loadMusic(ui.listQuery);
}

onMounted(reload);
watch(() => [props.kind, ui.listQuery], reload);

async function addFolder() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    await library.addFolder(selected, props.kind === "video" ? "video" : "audio");
  }
}

function playItem(item: MediaItem) {
  const q = playback.mediaToQueue(sorted.value, props.kind === "video");
  const idx = sorted.value.findIndex((m) => m.id === item.id);
  void playback.playPath(item.path, {
    title: item.title,
    mediaId: item.id,
    isVideo: props.kind === "video",
    artworkPath: item.artwork_path,
    queue: q,
    queueIndex: Math.max(0, idx),
  });
}

function playFirst() {
  if (sorted.value[0]) playItem(sorted.value[0]);
}

const videoSortOpts = [
  ["title", "sort.title"],
  ["added", "sort.added"],
  ["duration", "sort.duration"],
  ["recent", "sort.recent"],
] as const;
const musicSortOpts = [
  ["title", "sort.title"],
  ["artist", "sort.artist"],
  ["album", "sort.album"],
  ["duration", "sort.duration"],
] as const;
</script>

<template>
  <div>
    <div class="title-row">
      <div>
        <h1>{{ kind === "video" ? t("nav.videos") : t("nav.music") }}</h1>
        <p class="sub">{{ sorted.length }} items</p>
      </div>
    </div>

    <div class="lib-bar">
      <label class="search">
        <AppIcon name="search" :size="16" />
        <input v-model="listQuery" type="search" :placeholder="t('library.search')" />
      </label>
      <button type="button" class="btn-ghost" @click="addFolder">
        {{ t("library.addFolder") }}
      </button>
      <button type="button" class="btn-primary" :disabled="!sorted.length" @click="playFirst">
        {{ t("library.play") }}
      </button>
      <div style="position: relative">
        <button
          type="button"
          class="icon-btn"
          :aria-label="t('library.sort')"
          @click="sortOpen = !sortOpen"
        >
          <AppIcon name="sort" :size="16" />
        </button>
        <div
          v-if="sortOpen"
          style="
            position: absolute;
            right: 0;
            top: 36px;
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 6px;
            z-index: 5;
            min-width: 160px;
          "
        >
          <button
            v-for="[id, label] in kind === 'video' ? videoSortOpts : musicSortOpts"
            :key="id"
            type="button"
            style="display: block; width: 100%; text-align: left; padding: 8px 10px; border-radius: 6px"
            @click="
              kind === 'video' ? (ui.videoSort = id as typeof ui.videoSort) : (ui.musicSort = id as typeof ui.musicSort);
              sortOpen = false;
            "
          >
            {{ t(label) }}
          </button>
        </div>
      </div>
    </div>

    <p v-if="!sorted.length" class="empty">
      {{ kind === "video" ? t("library.emptyVideos") : t("library.emptyMusic") }}
    </p>

    <table v-else class="media-table">
      <thead>
        <tr>
          <th>{{ kind === "video" ? t("library.title") : t("library.track") }}</th>
          <th style="width: 90px">{{ t("library.duration") }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="item in sorted" :key="item.id" @click="playItem(item)">
          <td>
            <div class="media-row">
              <span class="thumb" :class="{ square: kind === 'audio' }">
                <MediaThumb
                  :title="item.title"
                  :seed="item.path"
                  :path="item.path"
                  :media-id="item.id"
                  :artwork-path="item.artwork_path"
                  :wide="kind === 'video'"
                />
              </span>
              <span class="meta">
                <strong>{{ item.title }}</strong>
                <span>{{ item.path }}</span>
              </span>
            </div>
          </td>
          <td class="dur">{{ formatDuration(item.duration_ms) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
