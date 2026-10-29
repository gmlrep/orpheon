//! Library browse / scan commands.

use crate::state::AppState;
use library::{scan_root, MediaItem, MediaType, Playlist};
use serde::Serialize;
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderDto {
    pub name: String,
    pub path: String,
    pub count: i64,
    pub kind: String,
}

#[tauri::command]
pub fn list_media(
    state: State<'_, AppState>,
    media_type: Option<String>,
    query: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<MediaItem>, String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    let mt = media_type.as_deref().map(MediaType::parse);
    let mut items = db
        .list_media_limited(mt, limit)
        .map_err(|e| e.to_string())?;
    if let Some(q) = query {
        let q = q.trim().to_lowercase();
        if !q.is_empty() {
            items.retain(|m| {
                m.title.to_lowercase().contains(&q) || m.path.to_lowercase().contains(&q)
            });
        }
    }
    Ok(items)
}

#[tauri::command]
pub fn list_recent_media(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<MediaItem>, String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    db.list_recent(limit.unwrap_or(40))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_folders(
    state: State<'_, AppState>,
    kind: Option<String>,
) -> Result<Vec<FolderDto>, String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    let roots = db
        .list_scan_roots_with_kind(kind.as_deref())
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for (path, root_kind) in roots {
        let name = Path::new(&path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&path)
            .to_string();
        let count = db
            .count_media_under_prefix(&path)
            .map_err(|e| e.to_string())?;
        out.push(FolderDto {
            name,
            path,
            count,
            kind: root_kind,
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn add_library_folder(
    state: State<'_, AppState>,
    path: String,
    kind: String,
) -> Result<u64, String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    let n = scan_root(&db, Path::new(&path), &kind).map_err(|e| e.to_string())?;
    Ok(n as u64)
}

#[tauri::command]
pub fn rescan_library(state: State<'_, AppState>) -> Result<u64, String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    let roots = db.list_scan_roots(None).map_err(|e| e.to_string())?;
    let mut total = 0u64;
    for root in roots {
        // Re-scan as "video" for roots of unknown kind — list_scan_roots returns paths only.
        // Prefer kind from section: try video then audio if zero.
        let n = scan_root(&db, Path::new(&root), "video").map_err(|e| e.to_string())? as u64;
        total += n;
        if n == 0 {
            total += scan_root(&db, Path::new(&root), "audio").map_err(|e| e.to_string())? as u64;
        }
    }
    Ok(total)
}

#[tauri::command]
pub fn list_playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    db.list_playlists().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_playlist(state: State<'_, AppState>, name: String) -> Result<Playlist, String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    let id = db.create_playlist(&name).map_err(|e| e.to_string())?;
    Ok(Playlist { id, name })
}

#[tauri::command]
pub fn delete_playlist(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    db.delete_playlist(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_playlist(
    state: State<'_, AppState>,
    id: i64,
    name: String,
) -> Result<(), String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    db.rename_playlist(id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn playlist_items(state: State<'_, AppState>, id: i64) -> Result<Vec<MediaItem>, String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    db.list_playlist_media(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_to_playlist(
    state: State<'_, AppState>,
    playlist_id: i64,
    media_id: i64,
) -> Result<(), String> {
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    db.add_to_playlist(playlist_id, media_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn continue_watching(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let resumes = {
        let store = state
            .settings
            .lock()
            .map_err(|_| "settings lock".to_string())?;
        store.load_resume_positions().map_err(|e| e.to_string())?
    };
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
    let mut out = Vec::new();
    let mut pairs: Vec<_> = resumes.into_iter().collect();
    pairs.sort_by(|a, b| {
        b.1 .1
            .cmp(&a.1 .1)
            .then_with(|| {
                b.1 .0
                    .partial_cmp(&a.1 .0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    for (path, (pos, _time_ms)) in pairs.into_iter().take(24) {
        if let Some(item) = db
            .get_media_by_path(&path)
            .map_err(|e| e.to_string())?
        {
            out.push(serde_json::json!({
                "id": item.id,
                "title": item.title,
                "path": item.path,
                "progress": pos,
                "durationMs": item.duration_ms,
                "mediaType": item.media_type,
                "artworkPath": item.artwork_path,
            }));
        } else {
            let title = Path::new(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&path)
                .to_string();
            out.push(serde_json::json!({
                "id": null,
                "title": title,
                "path": path,
                "progress": pos,
                "durationMs": null,
                "mediaType": "video",
                "artworkPath": null,
            }));
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn touch_recent(state: State<'_, AppState>, media_id: i64) -> Result<(), String> {
    let settings = state.load_settings();
    if !settings.prefs.save_recent {
        return Ok(());
    }
    let db = state.library.lock().map_err(|_| "library lock".to_string())?;
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
    db.touch_recent(media_id).map_err(|e| e.to_string())
}
