import { defineStore } from "pinia";
import { shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SavedStream } from "../types";
import { sanitizeArtworkUrl } from "../utils/mediaUrl";

export const useStreamsStore = defineStore("streams", () => {
  const saved = shallowRef<SavedStream[]>([]);
  const loading = shallowRef(false);

  async function load() {
    loading.value = true;
    try {
      saved.value = await invoke<SavedStream[]>("list_saved_streams");
    } finally {
      loading.value = false;
    }
  }

  async function save(mrl: string, title?: string, artworkUrl?: string | null) {
    saved.value = await invoke<SavedStream[]>("save_stream", {
      mrl: mrl.trim(),
      title: title?.trim() || null,
      artworkUrl: sanitizeArtworkUrl(artworkUrl),
    });
  }

  async function remove(mrl: string) {
    saved.value = await invoke<SavedStream[]>("remove_saved_stream", { mrl });
  }

  function isSaved(mrl: string) {
    return saved.value.some((s) => s.mrl === mrl);
  }

  return { saved, loading, load, save, remove, isSaved };
});
