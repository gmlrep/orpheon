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
    // Windows canonicalize() yields `\\?\…`; strip before play/DB compares.
    let canon = strip_verbatim_prefix(canon);
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
        let root_canon = strip_verbatim_prefix(root_canon);
        if canon.starts_with(&root_canon) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Drop Windows `\\?\` / `\\?\UNC\` prefixes from [`std::fs::canonicalize`] results.
fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{rest}"));
        }
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::strip_verbatim_prefix;
    use std::path::PathBuf;

    #[test]
    fn strip_verbatim_drive_prefix() {
        let p = PathBuf::from(r"\\?\D:\ai-music\track.mp3");
        assert_eq!(
            strip_verbatim_prefix(p),
            PathBuf::from(r"D:\ai-music\track.mp3")
        );
    }

    #[test]
    fn strip_verbatim_unc_prefix() {
        let p = PathBuf::from(r"\\?\UNC\server\share\a.mp4");
        assert_eq!(
            strip_verbatim_prefix(p),
            PathBuf::from(r"\\server\share\a.mp4")
        );
    }

    #[test]
    fn leave_normal_path() {
        let p = PathBuf::from(r"D:\ai-music\track.mp3");
        assert_eq!(strip_verbatim_prefix(p.clone()), p);
    }
}
