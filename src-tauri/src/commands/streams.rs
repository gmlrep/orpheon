//! Saved network stream bookmarks.

use crate::state::AppState;
use settings::SavedStream;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[tauri::command]
pub fn list_saved_streams(state: State<'_, AppState>) -> Result<Vec<SavedStream>, String> {
    state
        .settings
        .lock()
        .map_err(|_| "settings lock".to_string())?
        .load_saved_streams()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_stream(
    state: State<'_, AppState>,
    mrl: String,
    title: Option<String>,
    artwork_url: Option<String>,
) -> Result<Vec<SavedStream>, String> {
    let mrl = mrl.trim().to_string();
    if mrl.is_empty() {
        return Err("Stream URL is empty".into());
    }
    #[cfg(feature = "vlc")]
    {
        crate::player_service::validate_network_mrl(&mrl).map_err(|e| e.to_string())?;
    }

    let existing = state
        .settings
        .lock()
        .map_err(|_| "settings lock".to_string())?
        .load_saved_streams()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|s| s.mrl == mrl);

    let title = title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .or_else(|| existing.as_ref().map(|s| s.title.clone()))
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| mrl.clone());
    let artwork_url = artwork_url
        .filter(|u| !u.trim().is_empty())
        .or_else(|| existing.and_then(|s| s.artwork_url));

    let stream = SavedStream {
        mrl,
        title,
        artwork_url,
        saved_at: now_secs(),
    };
    state
        .settings
        .lock()
        .map_err(|_| "settings lock".to_string())?
        .upsert_saved_stream(stream)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_saved_stream(
    state: State<'_, AppState>,
    mrl: String,
) -> Result<Vec<SavedStream>, String> {
    state
        .settings
        .lock()
        .map_err(|_| "settings lock".to_string())?
        .remove_saved_stream(mrl.trim())
        .map_err(|e| e.to_string())
}

/// Merge discovered stream metadata into a saved bookmark when present.
pub(crate) fn enrich_saved_stream_meta(
    state: &AppState,
    mrl: &str,
    title: &str,
    artwork_url: Option<&str>,
) {
    let Ok(store) = state.settings.lock() else {
        return;
    };
    let Ok(mut list) = store.load_saved_streams() else {
        return;
    };
    let Some(entry) = list.iter_mut().find(|s| s.mrl == mrl) else {
        return;
    };
    let mut changed = false;
    if !title.is_empty() && title != mrl && (entry.title == mrl || entry.title.is_empty()) {
        entry.title = title.to_string();
        changed = true;
    }
    if let Some(art) = artwork_url {
        if !art.is_empty() && entry.artwork_url.as_deref() != Some(art) {
            entry.artwork_url = Some(art.to_string());
            changed = true;
        }
    }
    if changed {
        entry.saved_at = now_secs();
        let _ = store.save_saved_streams(&list);
    }
}
