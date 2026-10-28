<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { storeToRefs } from "pinia";
import { useLibraryStore } from "../stores/library";
import { usePlaybackStore } from "../stores/playback";
import MediaThumb from "../components/MediaThumb.vue";
import type { MediaItem } from "../types";

const { t } = useI18n();
const library = useLibraryStore();
const playback = usePlaybackStore();
const { playlists } = storeToRefs(library);
const selectedId = ref<number | null>(null);
const items = ref<MediaItem[]>([]);

onMounted(() => {
  void library.loadPlaylists();
});

async function create() {
  const name = window.prompt("Playlist name");
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
</script>

<template>
  <div>
    <div class="title-row">
      <div>
        <h1>{{ t("playlists.title") }}</h1>
      </div>
      <button type="button" class="btn-primary" @click="create">{{ t("playlists.new") }}</button>
    </div>

    <p v-if="!playlists.length" class="empty">{{ t("playlists.empty") }}</p>

    <div v-else class="settings-layout">
      <div class="list-rows">
        <div v-for="p in playlists" :key="p.id" class="row" style="justify-content: space-between">
          <button type="button" style="flex: 1; text-align: left" @click="select(p.id)">
            <strong>{{ p.name }}</strong>
          </button>
          <button type="button" class="btn-ghost" @click="remove(p.id)">Delete</button>
        </div>
      </div>
      <div>
        <div v-if="selectedId == null" class="empty">Select a playlist</div>
        <div v-else class="list-rows">
          <button
            v-for="(item, idx) in items"
            :key="item.id"
            type="button"
            @click="
              playback.playPath(item.path, {
                title: item.title,
                mediaId: item.id,
                isVideo: item.media_type !== 'audio',
                artworkPath: item.artwork_path,
                queue: playback.mediaToQueue(items, item.media_type !== 'audio'),
                queueIndex: idx,
              })
            "
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
          <p v-if="!items.length" class="empty">Empty playlist</p>
        </div>
      </div>
    </div>
  </div>
</template>
