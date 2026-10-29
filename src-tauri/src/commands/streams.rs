//! Saved network stream bookmarks.

use crate::error::CommandError;
use crate::mrl::{sanitize_artwork_url, validate_network_mrl};
use crate::state::AppState;
use crate::time_util::now_unix_secs;
use settings::SavedStream;
use tauri::State;

#[tauri::command]
pub fn list_saved_streams(state: State<'_, AppState>) -> Result<Vec<SavedStream>, CommandError> {
    Ok(state.lock_settings()?.load_saved_streams()?)
}

#[tauri::command]
pub fn save_stream(
    state: State<'_, AppState>,
    mrl: String,
    title: Option<String>,
    artwork_url: Option<String>,
) -> Result<Vec<SavedStream>, CommandError> {
    let mrl = validate_network_mrl(&mrl)?.to_string();

    let existing = state
        .lock_settings()?
        .load_saved_streams()?
        .into_iter()
        .find(|s| s.mrl == mrl);

    let title = title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .or_else(|| existing.as_ref().map(|s| s.title.clone()))
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| mrl.clone());
    let artwork_url = sanitize_artwork_url(artwork_url.as_deref())
        .or_else(|| sanitize_artwork_url(existing.and_then(|s| s.artwork_url).as_deref()));

    let stream = SavedStream {
        mrl,
        title,
        artwork_url,
        saved_at: now_unix_secs(),
    };
    Ok(state.lock_settings()?.upsert_saved_stream(stream)?)
}

#[tauri::command]
pub fn remove_saved_stream(
    state: State<'_, AppState>,
    mrl: String,
) -> Result<Vec<SavedStream>, CommandError> {
    Ok(state.lock_settings()?.remove_saved_stream(mrl.trim())?)
}

/// Merge discovered stream metadata into a saved bookmark when present.
pub(crate) fn enrich_saved_stream_meta(
    state: &AppState,
    mrl: &str,
    title: &str,
    artwork_url: Option<&str>,
) {
    let art = sanitize_artwork_url(artwork_url);
    let Ok(store) = state.lock_settings() else {
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
    if let Some(ref art) = art {
        if entry.artwork_url.as_deref() != Some(art.as_str()) {
            entry.artwork_url = Some(art.clone());
            changed = true;
        }
    }
    if changed {
        entry.saved_at = now_unix_secs();
        let _ = store.save_saved_streams(&list);
    }
}
