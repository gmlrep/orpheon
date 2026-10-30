//! Library browse / scan commands.

use crate::error::CommandError;
use crate::state::AppState;
use library::{scan_root, LibraryDb, MediaItem, MediaType, Playlist};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderDto {
    pub name: String,
    pub path: String,
    pub count: i64,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueWatchingItem {
    pub id: Option<i64>,
    pub title: String,
    pub path: String,
    pub progress: f32,
    pub duration_ms: Option<i64>,
    pub media_type: MediaType,
    pub artwork_path: Option<String>,
}

/// Known-folder hint for onboarding (localized Music / Videos via OS APIs).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFolderHint {
    /// Absolute path from the OS known-folder API (localized folder name).
    pub path: String,
    /// `true` when the path exists and is a directory.
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultMediaFolders {
    pub music: Option<MediaFolderHint>,
    pub videos: Option<MediaFolderHint>,
}

fn folder_hint(path: Option<&Path>) -> Option<MediaFolderHint> {
    let path = path?;
    let path_buf = PathBuf::from(path);
    // Prefer canonical path when the folder exists (stable casing / junctions).
    let (display, valid) = if path_buf.is_dir() {
        let canon = path_buf
            .canonicalize()
            .unwrap_or(path_buf)
            .to_string_lossy()
            .into_owned();
        // Windows canonicalize may prefix `\\?\` — strip for UI / dialogs.
        let display = canon.strip_prefix(r"\\?\").unwrap_or(&canon).to_string();
        (display, true)
    } else {
        (path_buf.to_string_lossy().into_owned(), false)
    };
    Some(MediaFolderHint {
        path: display,
        valid,
    })
}

/// Resolve the current user's Music and Videos folders (language-independent).
#[tauri::command]
pub fn default_media_folders() -> DefaultMediaFolders {
    let Some(dirs) = directories::UserDirs::new() else {
        return DefaultMediaFolders {
            music: None,
            videos: None,
        };
    };
    DefaultMediaFolders {
        music: folder_hint(dirs.audio_dir()),
        videos: folder_hint(dirs.video_dir()),
    }
}

#[tauri::command]
pub fn list_media(
    state: State<'_, AppState>,
    media_type: Option<String>,
    query: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<MediaItem>, CommandError> {
    let db = state.lock_library()?;
    let mt = media_type.as_deref().map(MediaType::parse);
    Ok(db.list_media_filtered(mt, query.as_deref(), limit)?)
}

#[tauri::command]
pub fn list_recent_media(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<MediaItem>, CommandError> {
    let db = state.lock_library()?;
    Ok(db.list_recent(limit.unwrap_or(40))?)
}

#[tauri::command]
pub fn list_folders(
    state: State<'_, AppState>,
    kind: Option<String>,
) -> Result<Vec<FolderDto>, CommandError> {
    let db = state.lock_library()?;
    let roots = db.list_scan_roots_with_kind(kind.as_deref())?;
    let mut out = Vec::new();
    for (path, root_kind) in roots {
        let name = Path::new(&path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&path)
            .to_string();
        let count = db.count_media_under_prefix(&path)?;
        out.push(FolderDto {
            name,
            path,
            count,
            kind: root_kind,
        });
    }
    Ok(out)
}

/// Scan on a dedicated SQLite connection so the managed `library` mutex stays free.
#[tauri::command]
pub async fn add_library_folder(
    state: State<'_, AppState>,
    path: String,
    kind: String,
) -> Result<u64, CommandError> {
    let db_path = state.library_db_path();
    tauri::async_runtime::spawn_blocking(move || {
        let db = LibraryDb::open(&db_path)?;
        let n = scan_root(&db, Path::new(&path), &kind)?;
        Ok(n as u64)
    })
    .await
    .map_err(|e| CommandError::Join(e.to_string()))?
}

#[tauri::command]
pub async fn rescan_library(state: State<'_, AppState>) -> Result<u64, CommandError> {
    let db_path = state.library_db_path();
    // Snapshot roots under a short lock, then scan on a separate connection.
    let roots = {
        let db = state.lock_library()?;
        db.list_scan_roots_with_kind(None)?
    };
    tauri::async_runtime::spawn_blocking(move || {
        let db = LibraryDb::open(&db_path)?;
        let mut total = 0u64;
        for (root, kind) in roots {
            let kind = if kind.is_empty() {
                "video".into()
            } else {
                kind
            };
            let n = scan_root(&db, Path::new(&root), &kind)? as u64;
            total += n;
            if n == 0 && kind == "video" {
                total += scan_root(&db, Path::new(&root), "audio")? as u64;
            }
        }
        Ok(total)
    })
    .await
    .map_err(|e| CommandError::Join(e.to_string()))?
}

#[tauri::command]
pub fn list_playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, CommandError> {
    Ok(state.lock_library()?.list_playlists()?)
}

#[tauri::command]
pub fn create_playlist(state: State<'_, AppState>, name: String) -> Result<Playlist, CommandError> {
    let db = state.lock_library()?;
    let id = db.create_playlist(&name)?;
    Ok(Playlist { id, name })
}

#[tauri::command]
pub fn delete_playlist(state: State<'_, AppState>, id: i64) -> Result<(), CommandError> {
    Ok(state.lock_library()?.delete_playlist(id)?)
}

#[tauri::command]
pub fn rename_playlist(
    state: State<'_, AppState>,
    id: i64,
    name: String,
) -> Result<(), CommandError> {
    Ok(state.lock_library()?.rename_playlist(id, &name)?)
}

#[tauri::command]
pub fn playlist_items(state: State<'_, AppState>, id: i64) -> Result<Vec<MediaItem>, CommandError> {
    Ok(state.lock_library()?.list_playlist_media(id)?)
}

#[tauri::command]
pub fn add_to_playlist(
    state: State<'_, AppState>,
    playlist_id: i64,
    media_id: i64,
) -> Result<(), CommandError> {
    Ok(state
        .lock_library()?
        .add_to_playlist(playlist_id, media_id)?)
}

#[tauri::command]
pub fn continue_watching(
    state: State<'_, AppState>,
) -> Result<Vec<ContinueWatchingItem>, CommandError> {
    let resumes = {
        let store = state.lock_settings()?;
        store.load_resume_positions()?
    };
    let db = state.lock_library()?;
    let mut out = Vec::new();
    let mut pairs: Vec<_> = resumes.into_iter().collect();
    pairs.sort_by(|a, b| {
        b.1 .1.cmp(&a.1 .1).then_with(|| {
            b.1 .0
                .partial_cmp(&a.1 .0)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    for (path, (pos, _time_ms)) in pairs.into_iter().take(48) {
        if let Some(item) = db.get_media_by_path(&path)? {
            // Continue watching lists video only (audio is never resumed).
            if item.media_type != MediaType::Video {
                continue;
            }
            out.push(ContinueWatchingItem {
                id: Some(item.id),
                title: item.title,
                path: item.path,
                progress: pos,
                duration_ms: item.duration_ms,
                media_type: item.media_type,
                artwork_path: item.artwork_path,
            });
        } else {
            let ext = Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if library::is_audio_extension(&ext) {
                continue;
            }
            let title = Path::new(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&path)
                .to_string();
            out.push(ContinueWatchingItem {
                id: None,
                title,
                path,
                progress: pos,
                duration_ms: None,
                media_type: MediaType::Video,
                artwork_path: None,
            });
        }
        if out.len() >= 24 {
            break;
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn touch_recent(state: State<'_, AppState>, media_id: i64) -> Result<(), CommandError> {
    let settings = state.load_settings();
    if !settings.prefs.save_recent {
        return Ok(());
    }
    let db = state.lock_library()?;
    let filter = settings.prefs.recent_filter_regex.trim();
    if !filter.is_empty() {
        if let Ok(Some(item)) = db.get_media_by_id(media_id) {
            let needle = filter.to_ascii_lowercase();
            let path_l = item.path.to_ascii_lowercase();
            let title_l = item.title.to_ascii_lowercase();
            if path_l.contains(&needle) || title_l.contains(&needle) {
                return Ok(());
            }
        }
    }
    Ok(db.touch_recent(media_id)?)
}
