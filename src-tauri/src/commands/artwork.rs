//! Thumbnail / cover art commands.

use crate::artwork::{self, NO_ARTWORK_SENTINEL};
use crate::error::CommandError;
use crate::state::AppState;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn resolve_artwork(
    state: State<'_, AppState>,
    path: String,
    media_id: Option<i64>,
) -> Result<Option<String>, CommandError> {
    let media_path = PathBuf::from(&path);
    let id = media_id.unwrap_or_else(|| {
        // Stable hash fallback when id unknown.
        let mut h: i64 = 0;
        for b in path.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as i64);
        }
        h.abs()
    });

    // Prefer DB path if the file still exists; honor negative-cache sentinel.
    if let Some(mid) = media_id {
        if let Ok(db) = state.lock_library() {
            if let Ok(Some(item)) = db.get_media_by_id(mid) {
                if let Some(art) = item.artwork_path {
                    if art == NO_ARTWORK_SENTINEL {
                        return Ok(None);
                    }
                    if PathBuf::from(&art).is_file() {
                        return Ok(Some(art));
                    }
                    let _ = db.set_artwork_path(mid, "");
                }
            }
        }
    }

    let cache = state.artwork_cache_dir();
    let art = tauri::async_runtime::spawn_blocking(move || {
        artwork::ensure_artwork(&media_path, &cache, id)
    })
    .await
    .map_err(|e| CommandError::Join(e.to_string()))?;

    let Some(art) = art else {
        // Remember the miss so opening Music/Videos does not re-spawn ffmpeg forever.
        if let Some(mid) = media_id {
            if let Ok(db) = state.lock_library() {
                let _ = db.set_artwork_path(mid, NO_ARTWORK_SENTINEL);
            }
        }
        return Ok(None);
    };
    let art_str = art.to_string_lossy().to_string();

    if let Some(mid) = media_id {
        if let Ok(db) = state.lock_library() {
            let _ = db.set_artwork_path(mid, &art_str);
        }
    }
    Ok(Some(art_str))
}
