//! Typed settings and persistence.

use crate::prefs::Prefs;
use crate::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const RESUME_KEY: &str = "resume_positions";
const RESUME_CAP: usize = 200;
const SAVED_STREAMS_KEY: &str = "saved_streams";
const SAVED_STREAMS_CAP: usize = 100;

const INIT_SQL: &str = r"
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
";

/// Theme preference matching the Slint `Theme` global.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    /// Follow OS.
    System,
    /// Force light.
    Light,
    /// Force dark (product default — matches shell bootstrap).
    #[default]
    Dark,
}

/// Full application settings snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Theme preference.
    pub theme: ThemePreference,
    /// Last absolute softvol percent (`100` = 0 dB, up to `200`).
    pub volume: i32,
    /// Hardware decoding preference (mapped to libVLC when available).
    pub hw_accel: bool,
    /// Remember playback position.
    pub resume_playback: bool,
    /// Interface language code (e.g. `en`).
    pub language: String,
    /// Extra libVLC option strings (advanced).
    pub vlc_options: Vec<String>,
    /// First-run library onboarding completed (or skipped).
    #[serde(default)]
    pub onboarding_done: bool,
    /// VLC-aligned Tools → Preferences bag.
    #[serde(default)]
    pub prefs: Prefs,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // Matches frontend `DEFAULT_THEME` / `index.html` data-theme.
            theme: ThemePreference::Dark,
            volume: 80,
            hw_accel: true,
            resume_playback: true,
            language: "system".into(),
            vlc_options: Vec::new(),
            onboarding_done: false,
            prefs: Prefs::default(),
        }
    }
}

/// Settings persisted in SQLite.
pub struct SettingsStore {
    conn: Connection,
}

impl SettingsStore {
    /// Open settings DB at `path`.
    ///
    /// # Errors
    /// Returns [`SettingsError`] on I/O or SQLite failure.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(INIT_SQL)?;
        Ok(Self { conn })
    }

    /// In-memory store for tests.
    ///
    /// # Errors
    /// Returns [`SettingsError::Db`].
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(INIT_SQL)?;
        Ok(Self { conn })
    }

    /// Load settings (or defaults if empty).
    ///
    /// # Errors
    /// Returns [`SettingsError`] on DB/JSON failure.
    pub fn load(&self) -> Result<AppSettings> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = 'app'")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let json: String = row.get(0)?;
            Ok(serde_json::from_str(&json)?)
        } else {
            Ok(AppSettings::default())
        }
    }

    /// Persist settings.
    ///
    /// # Errors
    /// Returns [`SettingsError`] on DB/JSON failure.
    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        let json = serde_json::to_string(settings)?;
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES ('app', ?1)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![json],
        )?;
        Ok(())
    }

    /// Get a raw string setting.
    ///
    /// # Errors
    /// Returns [`SettingsError::Db`].
    pub fn get_raw(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Set a raw string setting.
    ///
    /// # Errors
    /// Returns [`SettingsError::Db`].
    pub fn set_raw(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Load continue-watching positions (`path → (relative 0..=1, time_ms)`).
    ///
    /// # Errors
    /// Returns [`SettingsError`] on DB/JSON failure.
    pub fn load_resume_positions(&self) -> Result<HashMap<String, (f32, i64)>> {
        let Some(json) = self.get_raw(RESUME_KEY)? else {
            return Ok(HashMap::new());
        };
        let entries: Vec<ResumeEntry> = serde_json::from_str(&json).unwrap_or_default();
        Ok(entries
            .into_iter()
            .map(|e| (e.path, (e.position, e.time_ms)))
            .collect())
    }

    /// Persist continue-watching positions (newest first, capped).
    ///
    /// # Errors
    /// Returns [`SettingsError`] on DB/JSON failure.
    pub fn save_resume_positions(&self, map: &HashMap<String, (f32, i64)>) -> Result<()> {
        let mut entries: Vec<ResumeEntry> = map
            .iter()
            .map(|(path, (position, time_ms))| ResumeEntry {
                path: path.clone(),
                position: *position,
                time_ms: *time_ms,
            })
            .collect();
        // Prefer entries with more progress when truncating.
        entries.sort_by(|a, b| {
            b.time_ms.cmp(&a.time_ms).then_with(|| {
                b.position
                    .partial_cmp(&a.position)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
        entries.truncate(RESUME_CAP);
        let json = serde_json::to_string(&entries)?;
        self.set_raw(RESUME_KEY, &json)
    }

    /// Load saved network streams (newest first).
    ///
    /// # Errors
    /// Returns [`SettingsError`] on DB/JSON failure.
    pub fn load_saved_streams(&self) -> Result<Vec<SavedStream>> {
        let Some(json) = self.get_raw(SAVED_STREAMS_KEY)? else {
            return Ok(Vec::new());
        };
        Ok(serde_json::from_str(&json).unwrap_or_default())
    }

    /// Persist saved network streams (newest first, capped).
    ///
    /// # Errors
    /// Returns [`SettingsError`] on DB/JSON failure.
    pub fn save_saved_streams(&self, streams: &[SavedStream]) -> Result<()> {
        let mut list = streams.to_vec();
        list.truncate(SAVED_STREAMS_CAP);
        let json = serde_json::to_string(&list)?;
        self.set_raw(SAVED_STREAMS_KEY, &json)
    }

    /// Insert or update a saved stream (moves to front).
    ///
    /// # Errors
    /// Returns [`SettingsError`] on DB/JSON failure.
    pub fn upsert_saved_stream(&self, stream: SavedStream) -> Result<Vec<SavedStream>> {
        let mut list = self.load_saved_streams()?;
        list.retain(|s| s.mrl != stream.mrl);
        list.insert(0, stream);
        self.save_saved_streams(&list)?;
        Ok(list)
    }

    /// Remove a saved stream by MRL.
    ///
    /// # Errors
    /// Returns [`SettingsError`] on DB/JSON failure.
    pub fn remove_saved_stream(&self, mrl: &str) -> Result<Vec<SavedStream>> {
        let mut list = self.load_saved_streams()?;
        list.retain(|s| s.mrl != mrl);
        self.save_saved_streams(&list)?;
        Ok(list)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ResumeEntry {
    path: String,
    position: f32,
    time_ms: i64,
}

/// A bookmarked network stream (MRL + optional display metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedStream {
    /// Stream URL / MRL.
    pub mrl: String,
    /// Display title (user label or discovered meta).
    pub title: String,
    /// Optional cover / station artwork URL or local path.
    #[serde(default)]
    pub artwork_url: Option<String>,
    /// Unix epoch seconds when saved / last updated.
    pub saved_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_load_should_roundtrip_volume_and_theme() {
        let store = SettingsStore::open_in_memory().expect("open");
        let s = AppSettings {
            volume: 42,
            theme: ThemePreference::Dark,
            ..AppSettings::default()
        };
        store.save(&s).expect("save");
        let loaded = store.load().expect("load");
        assert_eq!(loaded.volume, 42);
        assert_eq!(loaded.theme, ThemePreference::Dark);
    }

    #[test]
    fn resume_positions_should_roundtrip() {
        let store = SettingsStore::open_in_memory().expect("open");
        let mut map = HashMap::new();
        map.insert("/a.mkv".into(), (0.42_f32, 12_000_i64));
        store.save_resume_positions(&map).expect("save");
        let loaded = store.load_resume_positions().expect("load");
        assert_eq!(loaded.get("/a.mkv").copied(), Some((0.42, 12_000)));
    }

    #[test]
    fn saved_streams_should_upsert_and_remove() {
        let store = SettingsStore::open_in_memory().expect("open");
        let list = store
            .upsert_saved_stream(SavedStream {
                mrl: "https://example.com/a.m3u8".into(),
                title: "A".into(),
                artwork_url: None,
                saved_at: 1,
            })
            .expect("upsert");
        assert_eq!(list.len(), 1);
        let list = store
            .upsert_saved_stream(SavedStream {
                mrl: "https://example.com/a.m3u8".into(),
                title: "A2".into(),
                artwork_url: Some("https://img/a.png".into()),
                saved_at: 2,
            })
            .expect("update");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "A2");
        let list = store
            .remove_saved_stream("https://example.com/a.m3u8")
            .expect("remove");
        assert!(list.is_empty());
    }
}
