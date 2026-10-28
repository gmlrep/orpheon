//! Thumbnail / cover art commands.

use crate::artwork;
use crate::state::AppState;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn resolve_artwork(
    state: State<'_, AppState>,
    path: String,
    media_id: Option<i64>,
) -> Result<Option<String>, String> {
    let media_path = PathBuf::from(&path);
    let id = media_id.unwrap_or_else(|| {
        // Stable hash fallback when id unknown.
        let mut h: i64 = 0;
        for b in path.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as i64);
        }
        h.abs()
    });

    // Prefer DB path if the file still exists; clear stale paths (e.g. from older apps).
    if let Some(mid) = media_id {
        if let Ok(db) = state.library.lock() {
            if let Ok(Some(item)) = db.get_media_by_id(mid) {
                if let Some(art) = item.artwork_path {
                    if PathBuf::from(&art).is_file() {
                        return Ok(Some(art));
                    }
                    let _ = db.set_artwork_path(mid, "");
                }
            }
        }
    }

    let cache = state.data_dir.join("artwork");
    let Some(art) = artwork::ensure_artwork(&media_path, &cache, id) else {
        return Ok(None);
    };
    let art_str = art.to_string_lossy().to_string();

    if let Some(mid) = media_id {
        if let Ok(db) = state.library.lock() {
            let _ = db.set_artwork_path(mid, &art_str);
        }
    }
    Ok(Some(art_str))
}
