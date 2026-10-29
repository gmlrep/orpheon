<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { storeToRefs } from "pinia";
import { useLibraryStore } from "../stores/library";
import { usePlaybackStore } from "../stores/playback";
import MediaThumb from "../components/MediaThumb.vue";
import type { MediaItem } from "../types";

defineOptions({ name: "PlaylistsView" });

const { t } = useI18n();
const library = useLibraryStore();
const playback = usePlaybackStore();
const { playlists, pending } = storeToRefs(library);
const selectedId = ref<number | null>(null);
const items = ref<MediaItem[]>([]);

onMounted(() => {
  void library.loadPlaylists();
});

async function create() {
  const name = window.prompt(t("playlists.namePrompt"));
  if (!name?.trim()) return;
  const p = await library.createPlaylist(name.trim());
  selectedId.value = p.id;
  items.value = [];
}

async function select(id: number) {
  selectedId.value = id;
  items.value = await library.playlistItems(id);
}

async function remove(id: number) {
  await library.deletePlaylist(id);
  if (selectedId.value === id) {
    selectedId.value = null;
    items.value = [];
  }
}

function playItem(item: MediaItem, idx: number) {
  void playback.playMediaItem(item, items.value, idx);
}
</script>

<template>
  <div>
    <div class="title-row">
      <div>
        <h1>{{ t("playlists.title") }}</h1>
      </div>
      <button type="button" class="btn-primary" @click="create">{{ t("playlists.new") }}</button>
    </div>

    <p v-if="pending.playlists && !playlists.length" class="empty">{{ t("common.loading") }}</p>
    <p v-else-if="!playlists.length" class="empty">{{ t("playlists.empty") }}</p>

    <div v-else class="settings-layout">
      <div class="list-rows">
        <div v-for="p in playlists" :key="p.id" class="playlist-row">
          <button type="button" class="playlist-select" @click="select(p.id)">
            <strong>{{ p.name }}</strong>
          </button>
          <button type="button" class="btn-ghost" @click="remove(p.id)">
            {{ t("playlists.delete") }}
          </button>
        </div>
      </div>
      <div>
        <div v-if="selectedId == null" class="empty">{{ t("playlists.select") }}</div>
        <div v-else class="list-rows">
          <button
            v-for="(item, idx) in items"
            :key="item.id"
            type="button"
            @click="playItem(item, idx)"
          >
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
              <span>{{ item.path }}</span>
            </span>
          </button>
          <p v-if="!items.length" class="empty">{{ t("playlists.emptyItems") }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.playlist-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}
.playlist-select {
  flex: 1;
  text-align: left;
}
</style>
