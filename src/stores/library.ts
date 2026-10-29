import { defineStore } from "pinia";
import { shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FolderDto, MediaItem, Playlist, ResumeItem } from "../types";
import { RECENT_LIST_LIMIT } from "../config/app";

export type VideoSort = "title" | "added" | "duration" | "recent";
export type MusicSort = "title" | "duration";

export const useLibraryStore = defineStore("library", () => {
  const videos = shallowRef<MediaItem[]>([]);
  const music = shallowRef<MediaItem[]>([]);
  const recent = shallowRef<MediaItem[]>([]);
  const folders = shallowRef<FolderDto[]>([]);
  const playlists = shallowRef<Playlist[]>([]);
  const resume = shallowRef<ResumeItem[]>([]);
  const loading = shallowRef(false);
  const pending = shallowRef<Record<string, boolean>>({});
  const listQuery = shallowRef("");
  const videoSort = shallowRef<VideoSort>("added");
  const musicSort = shallowRef<MusicSort>("title");

  function setPending(key: string, value: boolean) {
    pending.value = { ...pending.value, [key]: value };
  }

  function setListQuery(value: string) {
    listQuery.value = value;
  }

  function setVideoSort(value: VideoSort) {
    videoSort.value = value;
  }

  function setMusicSort(value: MusicSort) {
    musicSort.value = value;
  }

  async function loadHome() {
    loading.value = true;
    setPending("home", true);
    try {
      resume.value = await invoke<ResumeItem[]>("continue_watching");
      recent.value = await invoke<MediaItem[]>("list_recent_media", { limit: 20 });
      folders.value = await invoke<FolderDto[]>("list_folders", { kind: null });
      const audio = recent.value.filter((m) => m.media_type === "audio");
      if (audio.length === 0) {
        const sample = await invoke<MediaItem[]>("list_media", {
          mediaType: "audio",
          query: null,
          limit: 8,
        });
        recent.value = [...recent.value, ...sample];
      }
    } finally {
      loading.value = false;
      setPending("home", false);
    }
  }

  async function loadVideos(query = "") {
    setPending("videos", true);
    try {
      videos.value = await invoke<MediaItem[]>("list_media", {
        mediaType: "video",
        query: query || null,
      });
    } finally {
      setPending("videos", false);
    }
  }

  async function loadMusic(query = "") {
    setPending("music", true);
    try {
      music.value = await invoke<MediaItem[]>("list_media", {
        mediaType: "audio",
        query: query || null,
      });
    } finally {
      setPending("music", false);
    }
  }

  async function loadRecent() {
    setPending("recent", true);
    try {
      recent.value = await invoke<MediaItem[]>("list_recent_media", { limit: RECENT_LIST_LIMIT });
    } finally {
      setPending("recent", false);
    }
  }

  async function loadPlaylists() {
    setPending("playlists", true);
    try {
      playlists.value = await invoke<Playlist[]>("list_playlists");
    } finally {
      setPending("playlists", false);
    }
  }

  async function addFolder(path: string, kind: "video" | "audio") {
    await invoke("add_library_folder", { path, kind });
    if (kind === "video") await loadVideos(listQuery.value);
    else await loadMusic(listQuery.value);
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

  async function addToPlaylist(playlistId: number, mediaId: number) {
    await invoke("add_to_playlist", { playlistId, mediaId });
  }

  return {
    videos,
    music,
    recent,
    folders,
    playlists,
    resume,
    loading,
    pending,
    listQuery,
    videoSort,
    musicSort,
    setListQuery,
    setVideoSort,
    setMusicSort,
    loadHome,
    loadVideos,
    loadMusic,
    loadRecent,
    loadPlaylists,
    addFolder,
    createPlaylist,
    deletePlaylist,
    playlistItems,
    addToPlaylist,
  };
});
