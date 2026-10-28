<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { storeToRefs } from "pinia";
import { usePlaybackStore } from "../stores/playback";
import { useUiStore } from "../stores/ui";
import AppIcon from "../components/AppIcon.vue";
import SeekBar from "../components/SeekBar.vue";
import VolumeWedge from "../components/VolumeWedge.vue";
import { formatDuration } from "../utils/format";

const { t } = useI18n();
const playback = usePlaybackStore();
const ui = useUiStore();
const { session, playing, tracks, muted, coverPath } = storeToRefs(playback);
const { tracksOpen, trackTab, fullscreen, uiHidden } = storeToRefs(ui);

const stageEl = ref<HTMLElement | null>(null);
const controlsEl = ref<HTMLElement | null>(null);
const showRemaining = ref(false);
/** In fullscreen, controls start visible then auto-hide. */
const controlsReveal = ref(true);
let hideTimer = 0;
let raf = 0;
let unlistenResize: (() => void) | null = null;
let fsPoll = 0;
let cursorPoll = 0;

const isMusic = computed(() => !session.value.isVideo);
const chromeVisible = computed(
  () => (!fullscreen.value || controlsReveal.value) && !uiHidden.value,
);
const coverSrc = computed(() =>
  coverPath.value ? convertFileSrc(coverPath.value.replace(/\\/g, "/")) : null,
);

const durationLabel = computed(() => {
  if (!showRemaining.value) return formatDuration(session.value.lengthMs);
  const left = Math.max(0, session.value.lengthMs - session.value.timeMs);
  const s = formatDuration(left);
  return s === "—" ? s : `-${s}`;
});

const sheetTracks = computed(() => {
  if (!tracks.value) return [] as [number, string][];
  if (trackTab.value === "video") return tracks.value.video;
  if (trackTab.value === "audio") return tracks.value.audio;
  return tracks.value.spu;
});

const selectedId = computed(() => {
  if (!tracks.value) return -1;
  if (trackTab.value === "video") return tracks.value.selectedVideo;
  if (trackTab.value === "audio") return tracks.value.selectedAudio;
  return tracks.value.selectedSpu;
});

async function syncStage() {
  if (!stageEl.value) return;
  await nextTick();
  // Fullscreen: HWND always fills the stage so aspect/letterbox never jumps.
  const box = (fullscreen.value ? stageEl.value.closest(".player-stage") : null) as
    | HTMLElement
    | null;
  const target = box ?? stageEl.value;
  const rect = target.getBoundingClientRect();
  const scale = window.devicePixelRatio || 1;
  const width = Math.max(1, Math.round(rect.width));
  const height = Math.max(1, Math.round(rect.height));

  await invoke("update_stage_rect", {
    rect: {
      x: Math.round(rect.left),
      y: Math.round(rect.top),
      width,
      height,
      visible: !isMusic.value,
      scaleFactor: scale,
    },
  });

  // Only punch holes while chrome is visible (opaque controls cover the hole).
  // Never cut a hover strip when chrome is hidden — that clips video paint.
  const holes: number[][] = [];
  if (controlsEl.value && chromeVisible.value) {
    const c = controlsEl.value.getBoundingClientRect();
    holes.push([
      Math.round((c.left - rect.left) * scale),
      Math.round((c.top - rect.top) * scale),
      Math.round(c.width * scale),
      Math.round(c.height * scale),
      0,
    ]);
  }
  const sheet = document.querySelector(".tracks-sheet.is-open") as HTMLElement | null;
  if (sheet && chromeVisible.value) {
    const s = sheet.getBoundingClientRect();
    holes.push([
      Math.round((s.left - rect.left) * scale),
      Math.round((s.top - rect.top) * scale),
      Math.round(s.width * scale),
      Math.round(s.height * scale),
      8,
    ]);
  }
  await invoke("set_overlay_holes", { holes });
}

function scheduleSync() {
  cancelAnimationFrame(raf);
  raf = requestAnimationFrame(() => {
    void syncStage();
  });
}

function revealControls() {
  controlsReveal.value = true;
  if (uiHidden.value) ui.uiHidden = false;
  window.clearTimeout(hideTimer);
  if (fullscreen.value && !tracksOpen.value) {
    hideTimer = window.setTimeout(() => {
      controlsReveal.value = false;
      scheduleSync();
    }, 2200);
  }
  scheduleSync();
}

function onStageMove() {
  if (fullscreen.value) revealControls();
}

async function toggleFullscreen() {
  const win = getCurrentWindow();
  const next = !(await win.isFullscreen());
  await win.setFullscreen(next);
  ui.setFullscreen(next);
  controlsReveal.value = true;
  if (next) revealControls();
  await nextTick();
  scheduleSync();
}

function openTab(tab: "video" | "audio" | "subs") {
  ui.openTrackTab(tab);
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
  scheduleSync();
  window.addEventListener("resize", scheduleSync);
  void playback.refreshTracks();
  if (!session.value.isVideo) {
    void playback.refreshCover(session.value.path, session.value.mediaId);
  }

  const win = getCurrentWindow();
  ui.setFullscreen(await win.isFullscreen());
  unlistenResize = await win.onResized(() => scheduleSync());

  fsPoll = window.setInterval(async () => {
    try {
      const fs = await win.isFullscreen();
      if (fs !== ui.fullscreen) {
        ui.setFullscreen(fs);
        controlsReveal.value = true;
        if (fs) revealControls();
        scheduleSync();
      }
    } catch {
      /* ignore */
    }
  }, 500);

  // Poll OS cursor for bottom strip while chrome is hidden (no HWND region hole).
  cursorPoll = window.setInterval(async () => {
    if (!fullscreen.value || controlsReveal.value || tracksOpen.value) return;
    try {
      const near = await invoke<boolean>("cursor_in_stage_bottom", { stripCssPx: 72 });
      if (near) revealControls();
    } catch {
      /* ignore */
    }
  }, 120);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", scheduleSync);
  window.clearTimeout(hideTimer);
  window.clearInterval(fsPoll);
  window.clearInterval(cursorPoll);
  cancelAnimationFrame(raf);
  unlistenResize?.();
  void playback.tearDownVideoSurface();
});

watch(
  () => [ui.sidebarMode, tracksOpen.value, fullscreen.value, controlsReveal.value, uiHidden.value],
  () => scheduleSync(),
);
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
      @dblclick="toggleFullscreen"
      @mousemove="onStageMove"
    >
      <div v-if="isMusic" class="music-stage">
        <img v-if="coverSrc" class="music-art" :src="coverSrc" alt="" />
        <div v-else class="music-art music-art--empty" />
        <strong>{{ session.title }}</strong>
        <span>{{ session.subtitle }}</span>
      </div>
    </div>

    <div
      v-if="tracksOpen && chromeVisible"
      class="tracks-sheet is-open"
      role="dialog"
      :aria-label="t('player.tracks')"
    >
      <div class="tracks-tabs">
        <button
          v-if="!isMusic"
          type="button"
          :aria-selected="trackTab === 'video'"
          @click="openTab('video')"
        >
          {{ t("player.video") }}
        </button>
        <button type="button" :aria-selected="trackTab === 'audio'" @click="openTab('audio')">
          {{ t("player.audio") }}
        </button>
        <button
          v-if="!isMusic"
          type="button"
          :aria-selected="trackTab === 'subs'"
          @click="openTab('subs')"
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
          @click="playback.setTrack('spu', -1)"
        >
          {{ t("player.off") }}
        </button>
        <button
          v-for="[id, label] in sheetTracks"
          :key="id"
          type="button"
          class="track-row"
          :data-on="selectedId === id || undefined"
          @click="playback.setTrack(trackTab === 'subs' ? 'spu' : trackTab, id)"
        >
          {{ label }}
        </button>
        <p v-if="!sheetTracks.length && trackTab !== 'subs'" class="empty">
          {{ t("player.noTracks") }}
        </p>
      </div>
    </div>

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
            @click="playback.seekRelative(-10000)"
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
            @click="playback.seekRelative(10000)"
          >
            <AppIcon name="forward-10-sec" :size="18" />
          </button>
        </div>

        <span class="time elapsed">{{ formatDuration(session.timeMs) }}</span>
        <SeekBar
          :value="session.position"
          @seek="playback.seek($event)"
          @scrub-start="playback.beginScrub()"
          @scrub-end="playback.endScrub()"
        />
        <button type="button" class="time remaining" @click="showRemaining = !showRemaining">
          {{ durationLabel }}
        </button>

        <VolumeWedge
          :value="session.volume"
          :muted="muted"
          @change="playback.setVolume($event)"
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
  background: #0b0b0b;
  padding: 24px;
  text-align: center;
}
.music-art {
  width: min(280px, 40vw);
  height: min(280px, 40vw);
  object-fit: cover;
  border-radius: 8px;
  background: #151515;
}
.music-art--empty {
  background: radial-gradient(circle at 40% 35%, #2a1c0e, #151515 70%);
}
.music-stage strong {
  font-size: 18px;
  font-weight: 600;
}
.music-stage span {
  color: var(--muted);
  font-size: 13px;
}
</style>
