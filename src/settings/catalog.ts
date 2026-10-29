import { PREF_OPTIONS } from "./prefOptions";

export type SettingsCatId =
  | "general"
  | "interface"
  | "privacy"
  | "controls"
  | "playback"
  | "video"
  | "audio"
  | "subtitles"
  | "library"
  | "network"
  | "performance"
  | "recording"
  | "extensions";

export type PathDialogFilter = { name: string; extensions: string[] };

export type SettingsRow =
  | { type: "toggle"; key: string; label: string; hint?: string }
  | { type: "select"; key: string; label: string; hint?: string; options?: string[] }
  | { type: "text"; key: string; label: string; hint?: string; placeholder?: string }
  | {
      type: "path";
      key: string;
      label: string;
      hint?: string;
      placeholder?: string;
      mode: "directory" | "file";
      filters?: PathDialogFilter[];
    }
  | { type: "theme"; label: string; hint?: string }
  | { type: "language"; label: string; hint?: string }
  | { type: "accent"; label: string; hint?: string }
  | { type: "button"; label: string; hint?: string; action: string; buttonLabel: string };

export interface SettingsSection {
  title?: string;
  rows: SettingsRow[];
  advanced?: boolean;
}

export interface SettingsPage {
  id: SettingsCatId;
  title: string;
  sections: SettingsSection[];
}

export const SETTINGS_NAV: {
  sectionKey: string;
  items: { id: SettingsCatId; labelKey: string }[];
}[] = [
  {
    sectionKey: "settings.navApp",
    items: [
      { id: "general", labelKey: "settings.general" },
      { id: "interface", labelKey: "settings.interface" },
      { id: "privacy", labelKey: "settings.privacy" },
      { id: "controls", labelKey: "settings.controls" },
    ],
  },
  {
    sectionKey: "settings.navMedia",
    items: [
      { id: "playback", labelKey: "settings.playback" },
      { id: "video", labelKey: "settings.video" },
      { id: "audio", labelKey: "settings.audio" },
      { id: "subtitles", labelKey: "settings.subtitles" },
      { id: "library", labelKey: "settings.library" },
    ],
  },
  {
    sectionKey: "settings.navSystem",
    items: [
      { id: "network", labelKey: "settings.network" },
      { id: "performance", labelKey: "settings.performance" },
      { id: "recording", labelKey: "settings.recording" },
      { id: "extensions", labelKey: "settings.extensions" },
    ],
  },
];

/** i18n keys for page headings (fallback to catalog title). */
export const SETTINGS_PAGE_TITLE_KEYS: Record<SettingsCatId, string> = {
  general: "settings.pages.general",
  interface: "settings.pages.interface",
  privacy: "settings.pages.privacy",
  controls: "settings.pages.controls",
  playback: "settings.pages.playback",
  video: "settings.pages.video",
  audio: "settings.pages.audio",
  subtitles: "settings.pages.subtitles",
  library: "settings.pages.library",
  network: "settings.pages.network",
  performance: "settings.pages.performance",
  recording: "settings.pages.recording",
  extensions: "settings.pages.extensions",
};

const t = (key: string, label: string, hint?: string): SettingsRow => ({
  type: "toggle",
  key,
  label,
  hint,
});
const s = (key: string, label: string, hint?: string, options?: string[]): SettingsRow => ({
  type: "select",
  key,
  label,
  hint,
  options: options ?? PREF_OPTIONS[key],
});
const v = (key: string, label: string, hint?: string, placeholder?: string): SettingsRow => ({
  type: "text",
  key,
  label,
  hint,
  placeholder,
});
const p = (
  key: string,
  label: string,
  mode: "directory" | "file",
  hint?: string,
  placeholder?: string,
  filters?: PathDialogFilter[],
): SettingsRow => ({
  type: "path",
  key,
  label,
  mode,
  hint,
  placeholder,
  filters,
});
const b = (action: string, label: string, buttonLabel: string, hint?: string): SettingsRow => ({
  type: "button",
  action,
  label,
  buttonLabel,
  hint,
});

export const SETTINGS_PAGES: Record<SettingsCatId, SettingsPage> = {
  general: {
    id: "general",
    title: "Global Settings",
    sections: [
      {
        title: "Appearance",
        rows: [
          { type: "language", label: "Interface language" },
          { type: "theme", label: "Theme" },
          t("reduce_motion", "Reduce motion", "Minimize animations and transitions."),
          { type: "accent", label: "Accent color" },
        ],
      },
      {
        title: "Startup",
        rows: [
          t(
            "confirm_dialogs",
            "Show confirmation and error dialogs",
            "Show a dialog box when user input is required, and show unimportant errors and warnings.",
          ),
          t(
            "auto_update",
            "Check for updates automatically",
            "Activate the automatic notification of new versions of the software. It runs once every two weeks.",
          ),
        ],
      },
      {
        title: "Diagnostics",
        rows: [
          t(
            "dev_mode",
            "Developer mode",
            "Raise verbosity for diagnostics (0=errors and standard messages, 1=warnings, 2=debug).",
          ),
          t("save_log", "Save a log", "Log all VLC messages to a text file."),
          p(
            "log_path",
            "Log file",
            "file",
            "Specify the log filename.",
            "—",
            [
              { name: "Log", extensions: ["log", "txt"] },
              { name: "All", extensions: ["*"] },
            ],
          ),
          s(
            "log_verbose",
            "Log details",
            "Select the logging verbosity, or default to the same verbosity given by --verbose.",
          ),
        ],
      },
      {
        title: "Advanced",
        advanced: true,
        rows: [
          t(
            "show_minor_warnings",
            "Show minor warnings",
            "Show unimportant errors and warnings in dialogs.",
          ),
          v(
            "extra_cmdline",
            "Additional command line options",
            "Extra libVLC options applied at engine startup (space-separated). Restart the app after changing.",
            "--option",
          ),
          s("log_format", "Log format", "Format of the log file (text or HTML)."),
          t(
            "collect_stats",
            "Collect playback statistics",
            "Locally collect miscellaneous statistics about the playing media.",
          ),
        ],
      },
    ],
  },
  interface: {
    id: "interface",
    title: "Interface Settings",
    sections: [
      {
        title: "Skin",
        rows: [
          t("use_custom_skin", "Use a custom skin", "Not wired yet — stored for future use."),
          p(
            "skin_file",
            "Skin file",
            "file",
            "Path to a custom skin file.",
            "—",
            [
              { name: "VLC skins", extensions: ["vlt", "wsz"] },
              { name: "All", extensions: ["*"] },
            ],
          ),
        ],
      },
      {
        title: "Window",
        rows: [
          t(
            "embed_video",
            "Play video inside main window",
            "Embed the video in the main window instead of opening a separate video window.",
          ),
          s(
            "resize_to_video",
            "Resize window to fit video",
            "The interface resizes to the native video size, or the video fits the interface size.",
          ),
          t("remember_window_geometry", "Remember window size and position"),
          t(
            "keep_above",
            "Keep window above other windows",
            "Keep the player window above other windows while media is playing.",
          ),
          t(
            "show_decorations",
            "Show window decorations",
            "Enable the title bar. Disabling it removes the titlebar and moves window buttons into the interface.",
          ),
          t(
            "show_filename_title",
            "Show the file name in the title bar",
            "Show the name of the song or video in the controller window title.",
          ),
          s(
            "fullscreen_screen",
            "Open fullscreen on",
            "Screen number for fullscreen, instead of the same screen where the interface is.",
          ),
        ],
      },
      {
        title: "Playback Controls",
        rows: [
          t("start_compact", "Start in compact mode", "Start in minimal view with limited controls."),
          s("autohide_controls", "Auto-hide the controls"),
          s("autohide_after", "Hide after"),
          t(
            "show_fs_controls",
            "Show controls in fullscreen",
            "Show a controller in fullscreen mode.",
          ),
        ],
      },
      {
        title: "Notifications and System Tray",
        rows: [
          t(
            "systray_icon",
            "Show a system tray icon",
            "Show an icon in the systray allowing you to control VLC for basic actions.",
          ),
          t(
            "start_minimized_tray",
            "Start minimized to the tray",
            "VLC will start with just an icon in your taskbar.",
          ),
          s(
            "track_popup",
            "Show a popup when the track changes",
            "Show a notification popup with the artist and track name when the current playlist item changes, when VLC is minimized or hidden.",
          ),
          s("popup_duration", "Popup duration"),
        ],
      },
      {
        title: "Playlist",
        rows: [t("playlist_tree", "Show the playlist as a tree")],
      },
    ],
  },
  privacy: {
    id: "privacy",
    title: "Privacy Settings",
    sections: [
      {
        title: "History",
        rows: [
          t(
            "save_recent",
            "Save recently played items",
            "Save the recently played items in the menu.",
          ),
          t(
            "resume_playback",
            "Remember the playback position",
            "Continue playback from the last position the next time this item is opened.",
          ),
          v(
            "recent_filter_regex",
            "Never remember items matching",
            "Substring matched against path or title (case-insensitive).",
          ),
          b("clear_history", "Clear history now", "Clear"),
          t(
            "fetch_metadata_net",
            "Fetch titles and cover art from the internet",
            "Allow network access to look up titles and cover art when metadata is missing.",
          ),
        ],
      },
      {
        title: "Last.fm",
        rows: [
          t(
            "lastfm_enabled",
            "Submit played tracks to last.fm",
            "Not wired yet — stored for future use.",
          ),
          v("lastfm_user", "Last.fm username"),
          v("lastfm_url", "Last.fm submission URL"),
        ],
      },
    ],
  },
  controls: {
    id: "controls",
    title: "Mouse, Keyboard & Remote Controls",
    sections: [
      {
        title: "Keyboard Shortcuts",
        rows: [
          s("hotkey_profile", "Defaults"),
          b("edit_shortcuts", "Edit Keyboard Shortcuts", "Edit…"),
          b("reset_shortcuts", "Reset all shortcuts", "Reset"),
        ],
      },
      {
        title: "Skip Amounts",
        rows: [
          s("skip_very_short", "Very short skip"),
          s("skip_short", "Short skip"),
          s("skip_medium", "Medium skip"),
          s("skip_long", "Long skip"),
        ],
      },
      {
        title: "Mouse",
        rows: [
          s("mouse_single", "Single click on the video"),
          s("mouse_double", "Double click on the video"),
          s("mouse_wheel", "Scroll wheel"),
          s("mouse_hscroll", "Horizontal scroll"),
          t("mouse_reverse_scroll", "Reverse scroll direction"),
        ],
      },
      {
        title: "Remotes",
        rows: [t("http_intf", "Web remote control"), s("http_port", "Port")],
      },
    ],
  },
  playback: {
    id: "playback",
    title: "Playback Preferences",
    sections: [
      {
        title: "Startup",
        rows: [
          s(
            "continue_mode",
            "Continue where you left off",
            "Continue playback from where you left off when reopening media.",
          ),
          t("autoplay", "Start playing as soon as a file opens"),
          s("default_rate", "Default playback speed"),
          t("pause_on_audio_duck", "Pause when another app uses audio"),
        ],
      },
      {
        title: "Tracks",
        rows: [
          s("preferred_quality", "Preferred video quality"),
          s("menu_language", "Preferred menu language"),
          s("default_audio_track", "Default audio track"),
          s("default_spu_track", "Default subtitle track"),
        ],
      },
      {
        title: "Order",
        rows: [
          s("repeat_mode", "Repeat"),
          t("shuffle_default", "Shuffle"),
          t("stop_after_each", "Stop after each item"),
          s("on_finish", "When playback finishes"),
        ],
      },
      {
        title: "Seeking",
        rows: [
          t(
            "seek_precise",
            "Seek precisely",
            "Prefer more accurate seeking. This may be slower on some formats.",
          ),
          t("scrub_thumbnail", "Show thumbnail while scrubbing"),
        ],
      },
      {
        title: "Files & Discs",
        rows: [
          s("optical_drive", "Default optical drive"),
          s("avi_fix", "Repair damaged AVI files"),
          t("load_mkv_side", "Load matching .mkv files from the same folder"),
        ],
      },
    ],
  },
  video: {
    id: "video",
    title: "Video Defaults",
    sections: [
      {
        title: "Output",
        rows: [
          t("enable_video", "Enable video"),
          t("start_fullscreen", "Start videos in fullscreen"),
          t("disable_screensaver", "Stop the screensaver during playback"),
          t("desktop_wallpaper", "Play video as desktop wallpaper"),
        ],
      },
      {
        title: "Picture",
        rows: [
          s("hue", "Hue"),
          s("brightness", "Brightness"),
          s("contrast", "Contrast"),
          s("saturation", "Saturation"),
          s("gamma", "Gamma"),
          b("reset_picture", "Reset picture", "Reset"),
        ],
      },
      {
        title: "Deinterlacing",
        rows: [s("deinterlace", "Deinterlacing"), s("deinterlace_mode", "Method")],
      },
      {
        title: "Framing",
        rows: [
          s("aspect_ratio", "Aspect ratio"),
          s("crop", "Crop"),
          t("scale_to_window", "Scale video to the window"),
          s("zoom", "Zoom"),
          s("video_position", "Position in the window"),
        ],
      },
      {
        title: "Snapshots",
        rows: [
          p("snapshot_dir", "Save to", "directory", undefined, "Pictures"),
          v("snapshot_prefix", "File name prefix", undefined, "vlcsnap"),
          s("snapshot_format", "Format"),
          s("snapshot_size", "Size", "Keep the original size, or scale the snapshot."),
          t("snapshot_sequential", "Number files sequentially"),
          t("snapshot_preview", "Show a preview after capture"),
        ],
      },
    ],
  },
  audio: {
    id: "audio",
    title: "Audio Defaults",
    sections: [
      {
        title: "Output",
        rows: [
          t("enable_audio", "Enable audio"),
          s("audio_device", "Output device"),
          s("speaker_setup", "Speaker setup"),
        ],
      },
      {
        title: "Volume",
        rows: [
          s(
            "max_volume",
            "Maximum volume",
            "Maximum volume displayed (can allow amplification above 100%).",
          ),
          s("startup_volume_mode", "Startup volume"),
          s("startup_volume_pct", "Always starts at:"),
          s("volume_step", "Volume step"),
        ],
      },
      {
        title: "Loudness",
        rows: [
          t(
            "normalize",
            "Normalize loudness",
            "Normalize volume across tracks. If average power over recent buffers is higher than the max level, volume is reduced.",
          ),
          s("normalize_target", "Target level"),
          t("night_mode", "Night mode"),
          s("replay_gain", "Replay gain"),
          s("replay_gain_preamp", "Replay gain preamp", "Set the global gain in dB (−20 … 20)."),
        ],
      },
      {
        title: "Effects",
        rows: [
          t("equalizer_enabled", "Equalizer", "Enable the 10-band graphic equalizer."),
          t(
            "eq_two_pass",
            "Two-pass",
            "Filter the audio twice. This provides a more intense equalizer effect.",
          ),
          s(
            "spatial_mode",
            "Spatial audio mode",
            "Defines the virtual surface of the room emulated by the spatializer.",
          ),
          t("stereo_widen", "Stereo widening", "Width of the virtual room used by the spatializer."),
          t("compressor", "Compressor"),
          s("pitch_shift", "Pitch shift"),
          t("chorus", "Chorus/flanger"),
          t("karaoke", "Karaoke"),
        ],
      },
      {
        title: "Tracks",
        rows: [s("audio_language", "Preferred audio language"), s("audio_delay", "Audio delay")],
      },
      {
        title: "Visualization",
        rows: [s("visualizer", "Visualizer")],
      },
    ],
  },
  subtitles: {
    id: "subtitles",
    title: "Subtitle Defaults",
    sections: [
      {
        title: "Tracks & Languages",
        rows: [
          s("spu_language", "Preferred language"),
          s("spu_encoding", "Text encoding"),
        ],
      },
      {
        title: "Loading Subtitles",
        rows: [
          t(
            "spu_autoload",
            "Load subtitle files automatically",
            "Automatically load subtitle files that match the media file name in the same folder.",
          ),
          s("spu_fuzzy", "File name matching"),
        ],
      },
      {
        title: "Appearance",
        rows: [
          s("spu_font", "Font"),
          s("spu_size_mode", "Size values"),
          s("spu_font_size", "Font size"),
          t("spu_bold", "Bold"),
          s("spu_color", "Text color"),
          s("spu_opacity", "Text opacity"),
          s("spu_outline", "Outline thickness"),
          s("spu_outline_color", "Outline color"),
          t("spu_shadow", "Shadow"),
          s("spu_shadow_color", "Shadow color"),
          s("spu_shadow_dist", "Shadow distance"),
          s("spu_background", "Background"),
          s("spu_vpos", "Vertical position"),
        ],
      },
      {
        title: "Timing",
        rows: [s("spu_delay", "Subtitle delay")],
      },
      {
        title: "On-Screen Messages",
        rows: [
          t("osd_shortcuts", "Show feedback for shortcuts"),
          s("osd_duration", "Show feedback for"),
          t("osd_filename", "Show filename when a video starts"),
          s("osd_filename_dur", "Show filename for"),
        ],
      },
    ],
  },
  library: {
    id: "library",
    title: "Playlists, Folders & Files",
    sections: [
      {
        title: "Opening Files",
        rows: [
          t(
            "single_instance",
            "Only allow one player window",
            "Not wired yet — stored for future use.",
          ),
          t("fm_association", "Apply to files opened from the file manager"),
          t("enqueue_from_fm", "Add to the playlist instead of replacing it"),
          s("folder_recurse", "When opening a folder"),
          t("show_hidden_files", "Show hidden files"),
        ],
      },
      {
        title: "Metadata & Artwork",
        rows: [
          s("metadata_source", "Fetch titles and artwork"),
          t(
            "use_media_library",
            "Use the media library",
            "Index media into the local media library database for browsing and search.",
          ),
        ],
      },
    ],
  },
  network: {
    id: "network",
    title: "Network Connections & Security",
    sections: [
      {
        title: "Buffering",
        rows: [
          s("cache_file", "Local files"),
          s("cache_disc", "Discs"),
          s("cache_network", "Network streams"),
          s("cache_capture", "Live capture"),
        ],
      },
      {
        title: "Proxy",
        rows: [
          s(
            "http_proxy",
            "HTTP proxy",
            "HTTP proxy to be used. It must be of the form http://[user@]myproxy.mydomain:myport/ ; if empty, the http_proxy environment variable will be tried.",
          ),
          v("http_proxy_pass", "HTTP proxy password", "Session only — not saved to disk."),
        ],
      },
      {
        title: "Security",
        rows: [
          t(
            "tls_verify",
            "Verify certificates against the system trust store",
            "Verify TLS certificates against the system trust store and reject invalid ones.",
          ),
          s("tls_ciphers", "Cipher preference"),
        ],
      },
    ],
  },
  performance: {
    id: "performance",
    title: "Performance Settings",
    sections: [
      {
        title: "Decoding",
        rows: [
          s(
            "hw_accel_mode",
            "Hardware-accelerated decoding",
            "Use GPU hardware decoding when available. Takes effect the next time the app starts.",
          ),
          s(
            "postproc",
            "Video post-processing",
            "Takes effect the next time the app starts.",
          ),
          s(
            "skip_loop_filter",
            "Skip the H.264 loop filter",
            "Takes effect the next time the app starts.",
          ),
          s(
            "decode_threads",
            "Decoding threads",
            "Takes effect the next time the app starts.",
          ),
        ],
      },
      {
        title: "Smooth Playback",
        rows: [
          t("drop_late_frames", "Drop frames when the system can't keep up"),
          t("realtime_prio", "Give VLC real-time priority"),
          s("process_prio", "Process priority"),
          b(
            "rescan_plugins",
            "Rescan plugins",
            "Rescan",
            "Rescan the plugins cache so newly installed modules are discovered.",
          ),
        ],
      },
    ],
  },
  recording: {
    id: "recording",
    title: "Recording, Casting & Streaming Out",
    sections: [
      {
        rows: [
          p(
            "record_dir",
            "Save recordings to",
            "directory",
            "Directory where the records will be stored.",
            "Videos",
          ),
          t(
            "record_stream_copy",
            "Record without re-encoding when possible",
            "When possible, the input stream will be recorded instead of using the stream output module.",
          ),
        ],
      },
      {
        title: "Casting",
        rows: [t("cast_discover", "Discover cast devices"), s("cast_quality", "Cast quality")],
      },
      {
        title: "Streaming",
        rows: [
          t("stream_local", "Play locally while streaming"),
          t("stream_audio", "Include audio"),
          t("stream_video", "Include video"),
          t("stream_subs", "Include subtitles"),
          s("stream_preset", "Encoding preset"),
          s("stream_tuning", "Tuning"),
        ],
      },
    ],
  },
  extensions: {
    id: "extensions",
    title: "Add-ons & Scripts",
    sections: [
      {
        title: "Add-ons",
        rows: [
          b(
            "find_extensions",
            "Find more extensions",
            "Add",
            "Open the official VideoLAN add-ons site to find Lua extensions and other packages.",
          ),
          t(
            "addons_autoload",
            "Load add-ons on startup",
            "Automatically load installed add-ons when VLC starts.",
          ),
        ],
      },
    ],
  },
};

export function pageTitle(id: string): string {
  return SETTINGS_PAGES[id as SettingsCatId]?.title ?? id;
}

export function pageTitleKey(id: string): string {
  return SETTINGS_PAGE_TITLE_KEYS[id as SettingsCatId] ?? "settings.title";
}
