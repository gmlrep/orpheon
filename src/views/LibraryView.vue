<script setup lang="ts">
import { computed, onMounted, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { storeToRefs } from "pinia";
import { useLibraryStore, type MusicSort, type VideoSort } from "../stores/library";
import { usePlaybackStore } from "../stores/playback";
import AppIcon from "../components/AppIcon.vue";
import MediaThumb from "../components/MediaThumb.vue";
import { formatDuration } from "../utils/format";
import type { MediaItem } from "../types";
import { SEARCH_DEBOUNCE_MS } from "../config/app";

defineOptions({ name: "LibraryView" });

const props = defineProps<{ kind: "video" | "audio" }>();

const { t } = useI18n();
const library = useLibraryStore();
const playback = usePlaybackStore();
const { videos, music, listQuery, videoSort, musicSort, pending } = storeToRefs(library);
const sortOpen = shallowRef(false);

const items = computed(() => (props.kind === "video" ? videos.value : music.value));
const isLoading = computed(() =>
  props.kind === "video" ? !!pending.value.videos : !!pending.value.music,
);

const sorted = computed(() => {
  const copy = [...items.value];
  if (props.kind === "video") {
    const key = videoSort.value;
    if (key === "title") copy.sort((a, b) => a.title.localeCompare(b.title));
    else if (key === "duration")
      copy.sort((a, b) => (a.duration_ms ?? 0) - (b.duration_ms ?? 0));
    else copy.sort((a, b) => b.id - a.id);
  } else {
    const key = musicSort.value;
    if (key === "duration") copy.sort((a, b) => (a.duration_ms ?? 0) - (b.duration_ms ?? 0));
    else copy.sort((a, b) => a.title.localeCompare(b.title));
  }
  return copy;
});

let reloadGen = 0;
let debounceTimer = 0;

async function reload() {
  const gen = ++reloadGen;
  if (props.kind === "video") await library.loadVideos(listQuery.value);
  else await library.loadMusic(listQuery.value);
  if (gen !== reloadGen) return;
}

function scheduleReload() {
  window.clearTimeout(debounceTimer);
  debounceTimer = window.setTimeout(() => {
    void reload();
  }, SEARCH_DEBOUNCE_MS);
}

onMounted(() => {
  void reload();
});

watch(
  () => [props.kind, listQuery.value] as const,
  ([kind, query], prev) => {
    if (!prev) {
      void reload();
      return;
    }
    const [prevKind] = prev;
    if (kind !== prevKind) void reload();
    else if (query !== prev[1]) scheduleReload();
  },
);

async function addFolder() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    await library.addFolder(selected, props.kind === "video" ? "video" : "audio");
  }
}

function playItem(item: MediaItem) {
  void playback.playMediaItem(item, sorted.value);
}

function playFirst() {
  if (sorted.value[0]) playItem(sorted.value[0]);
}

function onSort(id: string) {
  if (props.kind === "video") library.setVideoSort(id as VideoSort);
  else library.setMusicSort(id as MusicSort);
  sortOpen.value = false;
}

const videoSortOpts = [
  ["title", "sort.title"],
  ["added", "sort.added"],
  ["duration", "sort.duration"],
  ["recent", "sort.recent"],
] as const;
const musicSortOpts = [
  ["title", "sort.title"],
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
        <input
          :value="listQuery"
          type="search"
          :placeholder="t('library.search')"
          @input="library.setListQuery(($event.target as HTMLInputElement).value)"
        />
      </label>
      <button type="button" class="btn-ghost" @click="addFolder">
        {{ t("library.addFolder") }}
      </button>
      <button type="button" class="btn-primary" :disabled="!sorted.length" @click="playFirst">
        {{ t("library.play") }}
      </button>
      <div class="sort-wrap">
        <button
          type="button"
          class="icon-btn"
          :aria-label="t('library.sort')"
          @click="sortOpen = !sortOpen"
        >
          <AppIcon name="sort" :size="16" />
        </button>
        <div v-if="sortOpen" class="sort-menu">
          <button
            v-for="[id, label] in kind === 'video' ? videoSortOpts : musicSortOpts"
            :key="id"
            type="button"
            class="sort-option"
            @click="onSort(id)"
          >
            {{ t(label) }}
          </button>
        </div>
      </div>
    </div>

    <p v-if="isLoading && !sorted.length" class="empty">{{ t("common.loading") }}</p>
    <p v-else-if="!sorted.length" class="empty">
      {{ kind === "video" ? t("library.emptyVideos") : t("library.emptyMusic") }}
    </p>

    <table v-else class="media-table">
      <thead>
        <tr>
          <th>{{ kind === "video" ? t("library.title") : t("library.track") }}</th>
          <th class="dur-col">{{ t("library.duration") }}</th>
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

<style scoped>
.sort-wrap {
  position: relative;
}
.sort-menu {
  position: absolute;
  right: 0;
  top: 36px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 6px;
  z-index: 5;
  min-width: 160px;
}
.sort-option {
  display: block;
  width: 100%;
  text-align: left;
  padding: 8px 10px;
  border-radius: 6px;
}
.dur-col {
  width: 90px;
}
</style>
