export type SidebarMode = "open" | "rail" | "hidden" | "sheet";

export type RouteName =
  | "home"
  | "videos"
  | "music"
  | "playlists"
  | "recent"
  | "stream"
  | "settings"
  | "player"
  | "eq";

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
  prefs: Record<string, unknown>;
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
