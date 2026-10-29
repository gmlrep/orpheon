<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { TrackSnapshot } from "../types";

const { tracks, trackTab, isMusic } = defineProps<{
  tracks: TrackSnapshot | null;
  trackTab: "video" | "audio" | "subs";
  isMusic: boolean;
}>();

const emit = defineEmits<{
  selectTab: [tab: "video" | "audio" | "subs"];
  setTrack: [kind: string, id: number];
}>();

const { t } = useI18n();

const sheetTracks = computed(() => {
  if (!tracks) return [] as [number, string][];
  if (trackTab === "video") return tracks.video;
  if (trackTab === "audio") return tracks.audio;
  return tracks.spu;
});

const selectedId = computed(() => {
  if (!tracks) return -1;
  if (trackTab === "video") return tracks.selectedVideo;
  if (trackTab === "audio") return tracks.selectedAudio;
  return tracks.selectedSpu;
});
</script>

<template>
  <div class="tracks-sheet is-open" role="dialog" :aria-label="t('player.tracks')">
    <div class="tracks-tabs">
      <button
        v-if="!isMusic"
        type="button"
        :aria-selected="trackTab === 'video'"
        @click="emit('selectTab', 'video')"
      >
        {{ t("player.video") }}
      </button>
      <button
        type="button"
        :aria-selected="trackTab === 'audio'"
        @click="emit('selectTab', 'audio')"
      >
        {{ t("player.audio") }}
      </button>
      <button
        v-if="!isMusic"
        type="button"
        :aria-selected="trackTab === 'subs'"
        @click="emit('selectTab', 'subs')"
      >
        {{ t("player.subs") }}
      </button>
    </div>
    <div class="tracks-body">
      <button
        v-if="trackTab === 'subs'"
        type="button"
        class="track-row"
        :data-on="selectedId < 0 || undefined"
        @click="emit('setTrack', 'spu', -1)"
      >
        {{ t("player.off") }}
      </button>
      <button
        v-for="[id, label] in sheetTracks"
        :key="id"
        type="button"
        class="track-row"
        :data-on="selectedId === id || undefined"
        @click="emit('setTrack', trackTab === 'subs' ? 'spu' : trackTab, id)"
      >
        {{ label }}
      </button>
      <p v-if="!sheetTracks.length && trackTab !== 'subs'" class="empty">
        {{ t("player.noTracks") }}
      </p>
    </div>
  </div>
</template>
