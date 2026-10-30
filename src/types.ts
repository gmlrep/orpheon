export type SidebarMode = "open" | "rail" | "hidden" | "sheet";

export type RouteName =
  "home" | "videos" | "music" | "playlists" | "recent" | "stream" | "settings" | "player" | "eq";

export interface SessionInfo {
  active: boolean;
  paused: boolean;
  title: string;
  subtitle: string;
  path: string;
  mediaId: number | null;
  isVideo: boolean;
  position: number;
  timeMs: number;
  lengthMs: number;
  volume: number;
  /** Cover / station art (`http(s)://` or local artwork-cache path). */
  artworkUrl?: string | null;
  /** Ask-mode resume position (0..=1), not yet applied. */
  resumeOffer?: number | null;
}

export interface SavedStream {
  mrl: string;
  title: string;
  artworkUrl?: string | null;
  savedAt: number;
}

export interface MediaItem {
  id: number;
  path: string;
  title: string;
  duration_ms: number | null;
  media_type: "video" | "audio" | "unknown";
  mtime: number | null;
  artwork_path: string | null;
}

export interface FolderDto {
  name: string;
  path: string;
  count: number;
  kind: string;
}

export interface Playlist {
  id: number;
  name: string;
}

export interface ResumeItem {
  id: number | null;
  title: string;
  path: string;
  progress: number;
  durationMs: number | null;
  mediaType: string;
  artworkPath: string | null;
}

export interface AppSettings {
  theme: "system" | "light" | "dark";
  volume: number;
  hw_accel: boolean;
  resume_playback: boolean;
  language: string;
  vlc_options: string[];
  /** First-run library folder onboarding completed or skipped. */
  onboarding_done?: boolean;
  /** VLC-aligned Tools → Preferences bag (see Rust `Prefs`). */
  prefs: Record<string, string | number | boolean | null | undefined>;
}

export interface MediaFolderHint {
  path: string;
  valid: boolean;
}

export interface DefaultMediaFolders {
  music: MediaFolderHint | null;
  videos: MediaFolderHint | null;
}

export interface PlaybackTick {
  session: SessionInfo;
  playing: boolean;
}

export interface TrackSnapshot {
  video: [number, string][];
  audio: [number, string][];
  spu: [number, string][];
  chapters: [number, string][];
  selectedVideo: number;
  selectedAudio: number;
  selectedSpu: number;
  selectedChapter: number;
}
