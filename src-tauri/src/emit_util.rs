//! Event emit helpers that surface failures in logs.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Emit an event; log a warning if delivery fails.
pub fn emit_logged(app: &AppHandle, event: &str, payload: impl Serialize + Clone) {
    if let Err(e) = app.emit(event, payload) {
        log::warn!("emit `{event}` failed: {e}");
    }
}
