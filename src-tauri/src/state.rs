//! Shared application state for Tauri commands.

use crate::player_handle::PlayerHandle;
use library::LibraryDb;
use serde::{Deserialize, Serialize};
use settings::{AppSettings, SettingsStore};
use std::path::PathBuf;
use std::sync::Mutex;

#[cfg(all(feature = "vlc", windows))]
use crate::video_embed::VideoEmbed;

/// Now-playing session metadata mirrored to the Vue NowBar.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub active: bool,
    pub paused: bool,
    pub title: String,
    pub subtitle: String,
    pub path: String,
    pub media_id: Option<i64>,
    pub is_video: bool,
    pub position: f32,
    pub time_ms: i64,
    pub length_ms: i64,
    pub volume: i32,
}

/// Playback tick payload emitted to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackTick {
    pub session: SessionInfo,
    pub playing: bool,
}

/// Stage rect in window client coordinates (CSS/logical → physical applied by caller).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub visible: bool,
    pub scale_factor: f64,
}

/// App-wide managed state.
pub struct AppState {
    pub player: PlayerHandle,
    pub library: Mutex<LibraryDb>,
    pub settings: Mutex<SettingsStore>,
    pub session: Mutex<SessionInfo>,
    pub data_dir: PathBuf,
    pub stage: Mutex<StageRect>,
    /// Deferred resume seek (0..=1) applied once media length is known.
    pub pending_seek: Mutex<Option<f32>>,
    #[cfg(all(feature = "vlc", windows))]
    pub video: Mutex<Option<VideoEmbed>>,
}

impl AppState {
    /// Open library + settings under `data_dir` and attach an optional player.
    pub fn new(data_dir: PathBuf, player: PlayerHandle) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let library = LibraryDb::open(&data_dir.join("library.sqlite"))?;
        let settings = SettingsStore::open(&data_dir.join("settings.sqlite"))?;
        Ok(Self {
            player,
            library: Mutex::new(library),
            settings: Mutex::new(settings),
            session: Mutex::new(SessionInfo::default()),
            data_dir,
            stage: Mutex::new(StageRect {
                scale_factor: 1.0,
                ..Default::default()
            }),
            pending_seek: Mutex::new(None),
            #[cfg(all(feature = "vlc", windows))]
            video: Mutex::new(None),
        })
    }

    pub fn load_settings(&self) -> AppSettings {
        self.settings
            .lock()
            .ok()
            .and_then(|s| s.load().ok())
            .unwrap_or_default()
    }
}
