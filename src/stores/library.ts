import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FolderDto, MediaItem, Playlist, ResumeItem } from "../types";

export const useLibraryStore = defineStore("library", () => {
  const videos = ref<MediaItem[]>([]);
  const music = ref<MediaItem[]>([]);
  const recent = ref<MediaItem[]>([]);
  const folders = ref<FolderDto[]>([]);
  const playlists = ref<Playlist[]>([]);
  const resume = ref<ResumeItem[]>([]);
  const loading = ref(false);

  async function loadHome() {
    loading.value = true;
    try {
      resume.value = await invoke<ResumeItem[]>("continue_watching");
      recent.value = await invoke<MediaItem[]>("list_recent_media", { limit: 20 });
      folders.value = await invoke<FolderDto[]>("list_folders", { kind: null });
      const audio = recent.value.filter((m) => m.media_type === "audio");
      if (audio.length === 0) {
        const allAudio = await invoke<MediaItem[]>("list_media", {
          mediaType: "audio",
          query: null,
        });
        recent.value = [...recent.value, ...allAudio.slice(0, 8)];
      }
    } finally {
      loading.value = false;
    }
  }

  async function loadVideos(query = "") {
    videos.value = await invoke<MediaItem[]>("list_media", {
      mediaType: "video",
      query: query || null,
    });
  }

  async function loadMusic(query = "") {
    music.value = await invoke<MediaItem[]>("list_media", {
      mediaType: "audio",
      query: query || null,
    });
  }

  async function loadRecent() {
    recent.value = await invoke<MediaItem[]>("list_recent_media", { limit: 100 });
  }

  async function loadPlaylists() {
    playlists.value = await invoke<Playlist[]>("list_playlists");
  }

  async function addFolder(path: string, kind: "video" | "audio") {
    await invoke("add_library_folder", { path, kind });
    if (kind === "video") await loadVideos();
    else await loadMusic();
    folders.value = await invoke<FolderDto[]>("list_folders", { kind });
  }

  async function createPlaylist(name: string) {
    const p = await invoke<Playlist>("create_playlist", { name });
    playlists.value = [...playlists.value, p];
    return p;
  }

  async function deletePlaylist(id: number) {
    await invoke("delete_playlist", { id });
    playlists.value = playlists.value.filter((p) => p.id !== id);
  }

  async function playlistItems(id: number) {
    return invoke<MediaItem[]>("playlist_items", { id });
  }

  return {
    videos,
    music,
    recent,
    folders,
    playlists,
    resume,
    loading,
    loadHome,
    loadVideos,
    loadMusic,
    loadRecent,
    loadPlaylists,
    addFolder,
    createPlaylist,
    deletePlaylist,
    playlistItems,
  };
});
