//! Playback path allowlisting (library roots / known media / Open File).

use crate::error::CommandError;
use crate::state::AppState;
use library::{is_media_extension, LibraryDb};
use std::path::{Path, PathBuf};

/// Ensure `path` is a real media file the app is allowed to open.
///
/// Allowed when the canonical file:
/// - sits under a registered library scan root, or
/// - is already indexed in the library DB, or
/// - exists as a media file (Open File / dialog) — still requires a known extension.
pub fn assert_playable_path(state: &AppState, path: &str) -> Result<PathBuf, CommandError> {
    let raw = PathBuf::from(path);
    if path.is_empty() || path.contains('\0') {
        return Err(CommandError::msg("invalid media path"));
    }
    let canon = raw
        .canonicalize()
        .map_err(|_| CommandError::msg(format!("media path not found: {path}")))?;
    if !canon.is_file() {
        return Err(CommandError::msg("media path is not a file"));
    }
    let ext = canon
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !is_media_extension(&ext) {
        return Err(CommandError::msg(format!(
            "unsupported media extension '.{ext}'"
        )));
    }

    let path_str = canon.to_string_lossy().to_string();
    let db = state.lock_library()?;
    if db.get_media_by_path(&path_str)?.is_some()
        || db.get_media_by_path(path)?.is_some()
        || under_scan_root(&db, &canon)?
    {
        return Ok(canon);
    }
    drop(db);

    // Open File outside the library — media extension + existing file already checked.
    log::debug!("play_path outside library roots: {}", canon.display());
    Ok(canon)
}

fn under_scan_root(db: &LibraryDb, canon: &Path) -> Result<bool, CommandError> {
    let roots = db.list_scan_roots(None)?;
    for root in roots {
        let Ok(root_canon) = PathBuf::from(&root).canonicalize() else {
            // Compare loosely if canonicalize fails (missing drive, etc.).
            if canon.starts_with(Path::new(&root)) {
                return Ok(true);
            }
            continue;
        };
        if canon.starts_with(&root_canon) {
            return Ok(true);
        }
    }
    Ok(false)
}
