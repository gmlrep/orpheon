<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { storeToRefs } from "pinia";
import { usePlaybackStore } from "../stores/playback";
import { useSettingsStore } from "../stores/settings";
import { useUiStore } from "../stores/ui";
import AppIcon from "../components/AppIcon.vue";
import SeekBar from "../components/SeekBar.vue";
import VolumeWedge from "../components/VolumeWedge.vue";
import TracksSheet from "../components/TracksSheet.vue";
import { useCoverSrc } from "../composables/useCoverSrc";
import { useVideoStage } from "../composables/useVideoStage";
import { usePlayerChrome } from "../composables/usePlayerChrome";
import { formatDuration } from "../utils/format";

defineOptions({ name: "PlayerView" });

const { t } = useI18n();
const playback = usePlaybackStore();
const settings = useSettingsStore();
const ui = useUiStore();
const { session, playing, tracks, muted, coverPath, tracksOpen, trackTab } = storeToRefs(playback);
const { fullscreen } = storeToRefs(ui);

const skipShortMs = computed(() => settings.skipMs("short"));
const maxVolume = computed(() => settings.maxVolume());

const stageEl = useTemplateRef<HTMLElement>("stageEl");
const controlsEl = useTemplateRef<HTMLElement>("controlsEl");
const showRemaining = shallowRef(false);

const isMusic = computed(() => !session.value.isVideo);
const coverSrc = useCoverSrc(coverPath);

const scheduleSyncRef = shallowRef<() => void>(() => {});

const { chromeVisible, revealControls, onStageMove, toggleFullscreen } = usePlayerChrome({
  scheduleSync: () => scheduleSyncRef.value(),
  tracksOpen,
});

function runMouseAction(action: string) {
  switch (action) {
    case "Play / Pause":
      void playback.playPause();
      break;
    case "Fullscreen":
    case "Toggle fullscreen":
      void toggleFullscreen();
      break;
    case "Ignore":
    default:
      break;
  }
}

let clickTimer = 0;
function onStageClick() {
  window.clearTimeout(clickTimer);
  clickTimer = window.setTimeout(() => {
    runMouseAction(settings.mouseAction("single"));
  }, 220);
}

function onStageDblClick(e: MouseEvent) {
  e.preventDefault();
  window.clearTimeout(clickTimer);
  runMouseAction(settings.mouseAction("double"));
}

function onStageWheel(e: WheelEvent) {
  if (!playback.session.active) return;
  const horizontal = Math.abs(e.deltaX) > Math.abs(e.deltaY);
  const mode = horizontal ? settings.mouseHScrollMode() : settings.mouseWheelMode();
  if (mode === "Ignore") return;
  e.preventDefault();
  let delta = horizontal ? e.deltaX : e.deltaY;
  if (settings.mouseReverseScroll()) delta = -delta;
  const sign = delta > 0 ? -1 : 1;
  if (mode === "Volume") {
    void playback.nudgeVolume(sign * settings.volumeStep());
  } else if (mode === "Seek") {
    void playback.seekRelative(sign * settings.skipMs("short"));
  }
}

const { scheduleSync } = useVideoStage({
  stageEl,
  controlsEl,
  isMusic,
  chromeVisible,
});
scheduleSyncRef.value = scheduleSync;

const durationLabel = computed(() => {
  if (!showRemaining.value) return formatDuration(session.value.lengthMs);
  const left = Math.max(0, session.value.lengthMs - session.value.timeMs);
  const s = formatDuration(left);
  return s === "—" ? s : `-${s}`;
});

function openTab(tab: "video" | "audio" | "subs") {
  playback.openTrackTab(tab);
  void playback.refreshTracks();
  revealControls();
}

onMounted(async () => {
  if (!isMusic.value) {
    try {
      await invoke("ensure_video_surface");
    } catch (e) {
      console.error(e);
    }
  }
  void playback.refreshTracks();
  if (!session.value.isVideo) {
    void playback.refreshCover(session.value.path, session.value.mediaId);
  }
});

onBeforeUnmount(() => {
  window.clearTimeout(clickTimer);
  void playback.tearDownVideoSurface();
});
</script>

<template>
  <div
    class="player-stage"
    :data-fs="fullscreen || undefined"
    :data-chrome="chromeVisible ? 'on' : 'off'"
  >
    <div
      ref="stageEl"
      class="stage-surface"
      @click="onStageClick"
      @dblclick="onStageDblClick"
      @wheel="onStageWheel"
      @mousemove="onStageMove"
    >
      <div v-if="isMusic" class="music-stage">
        <img v-if="coverSrc" class="music-art" :src="coverSrc" alt="" />
        <div v-else class="music-art music-art--empty" />
        <strong class="music-title">{{ session.title }}</strong>
        <span class="music-subtitle">{{ session.subtitle }}</span>
      </div>
    </div>

    <TracksSheet
      v-if="tracksOpen && chromeVisible"
      :tracks="tracks"
      :track-tab="trackTab"
      :is-music="isMusic"
      @select-tab="openTab"
      @set-track="(kind, id) => playback.setTrack(kind, id)"
    />

    <div
      ref="controlsEl"
      class="player-controls"
      :class="{ 'is-hidden': !chromeVisible }"
      @mouseenter="revealControls"
      @mousemove="revealControls"
    >
      <div class="controls-row">
        <div class="transport">
          <button
            v-if="isMusic"
            type="button"
            class="icon-btn"
            :aria-label="t('player.back10')"
            @click="playback.seekRelative(-skipShortMs)"
          >
            <AppIcon name="backward-10-sec" :size="18" />
          </button>
          <button
            type="button"
            class="icon-btn"
            :aria-label="t('player.previous')"
            @click="playback.previousTrack()"
          >
            <AppIcon name="previous" :size="18" />
          </button>
          <button
            type="button"
            class="play"
            :aria-label="playing ? t('player.pause') : t('player.play')"
            @click="playback.playPause()"
          >
            <AppIcon :name="playing ? 'pause' : 'play'" :size="16" />
          </button>
          <button
            type="button"
            class="icon-btn"
            :aria-label="t('player.next')"
            @click="playback.nextTrack()"
          >
            <AppIcon name="next" :size="18" />
          </button>
          <button
            v-if="isMusic"
            type="button"
            class="icon-btn"
            :aria-label="t('player.fwd10')"
            @click="playback.seekRelative(skipShortMs)"
          >
            <AppIcon name="forward-10-sec" :size="18" />
          </button>
        </div>

        <span class="time elapsed">{{ formatDuration(session.timeMs) }}</span>
        <SeekBar
          :model-value="session.position"
          @update:model-value="playback.seek($event)"
          @scrub-start="playback.beginScrub()"
          @scrub-end="playback.endScrub()"
        />
        <button type="button" class="time remaining" @click="showRemaining = !showRemaining">
          {{ durationLabel }}
        </button>

        <VolumeWedge
          :model-value="session.volume"
          :muted="muted"
          :max-volume="maxVolume"
          @update:model-value="playback.setVolume($event)"
          @toggle-mute="playback.toggleMute()"
        />

        <div class="utils">
          <button
            v-if="!isMusic"
            type="button"
            class="icon-btn"
            :data-on="(tracksOpen && trackTab === 'video') || undefined"
            :aria-label="t('player.video')"
            @click="openTab('video')"
          >
            <AppIcon name="video" :size="18" />
          </button>
          <button
            type="button"
            class="icon-btn"
            :data-on="(tracksOpen && trackTab === 'audio') || undefined"
            :aria-label="t('player.audio')"
            @click="openTab('audio')"
          >
            <AppIcon name="audio" :size="18" />
          </button>
          <button
            v-if="!isMusic"
            type="button"
            class="icon-btn"
            :data-on="(tracksOpen && trackTab === 'subs') || undefined"
            :aria-label="t('player.subs')"
            @click="openTab('subs')"
          >
            <AppIcon name="subtitles" :size="18" />
          </button>
          <button
            type="button"
            class="icon-btn"
            :aria-label="t('player.fullscreen')"
            @click="toggleFullscreen"
          >
            <AppIcon name="fullscreen" :size="18" />
          </button>
          <button
            type="button"
            class="icon-btn"
            :aria-label="t('player.minimize')"
            @click="playback.minimizePlayer()"
          >
            <AppIcon name="minimize" :size="18" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.player-controls.is-hidden {
  opacity: 0;
  pointer-events: none;
  transform: translateY(8px);
}
.player-controls {
  transition:
    opacity 160ms ease,
    transform 160ms ease;
}
.player-stage[data-fs] .stage-surface {
  position: absolute;
  inset: 0;
}
.music-stage {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  background: var(--player-void);
  padding: 24px;
  text-align: center;
}
.music-art {
  width: min(280px, 40vw);
  height: min(280px, 40vw);
  object-fit: cover;
  border-radius: 8px;
  background: var(--player-art-empty);
}
.music-art--empty {
  background: radial-gradient(
    circle at 40% 35%,
    var(--player-art-glow),
    var(--player-art-empty) 70%
  );
}
.music-title {
  font-size: 18px;
  font-weight: 600;
}
.music-subtitle {
  color: var(--muted);
  font-size: 13px;
}
</style>
