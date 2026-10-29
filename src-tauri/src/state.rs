//! Shared application state for Tauri commands.

use crate::error::CommandError;
use crate::player_handle::PlayerHandle;
use library::LibraryDb;
use serde::{Deserialize, Serialize};
use settings::{AppSettings, SettingsStore};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

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
    /// Cover / station art (`http(s)://` only for remote; local paths from artwork cache).
    #[serde(default)]
    pub artwork_url: Option<String>,
    /// When continue mode is Ask: offered resume position (0..=1), not yet applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_offer: Option<f32>,
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
        std::fs::create_dir_all(data_dir.join("artwork"))?;
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
        match self.lock_settings() {
            Ok(store) => store.load().unwrap_or_default(),
            Err(e) => {
                log::error!("{e}");
                AppSettings::default()
            }
        }
    }

    pub fn lock_library(&self) -> Result<MutexGuard<'_, LibraryDb>, CommandError> {
        self.library
            .lock()
            .map_err(|_| CommandError::Lock("library"))
    }

    pub fn lock_settings(&self) -> Result<MutexGuard<'_, SettingsStore>, CommandError> {
        self.settings
            .lock()
            .map_err(|_| CommandError::Lock("settings"))
    }

    pub fn lock_session(&self) -> Result<MutexGuard<'_, SessionInfo>, CommandError> {
        self.session
            .lock()
            .map_err(|_| CommandError::Lock("session"))
    }

    pub fn lock_pending_seek(&self) -> Result<MutexGuard<'_, Option<f32>>, CommandError> {
        self.pending_seek
            .lock()
            .map_err(|_| CommandError::Lock("pending_seek"))
    }

    pub fn lock_stage(&self) -> Result<MutexGuard<'_, StageRect>, CommandError> {
        self.stage.lock().map_err(|_| CommandError::Lock("stage"))
    }

    #[cfg(all(feature = "vlc", windows))]
    pub fn lock_video(&self) -> Result<MutexGuard<'_, Option<VideoEmbed>>, CommandError> {
        self.video.lock().map_err(|_| CommandError::Lock("video"))
    }

    /// Path to the on-disk SQLite library (for short-lived scan connections).
    pub fn library_db_path(&self) -> PathBuf {
        self.data_dir.join("library.sqlite")
    }

    pub fn artwork_cache_dir(&self) -> PathBuf {
        self.data_dir.join("artwork")
    }
}
