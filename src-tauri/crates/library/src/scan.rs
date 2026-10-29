//! Filesystem scanning for media files.

use crate::models::MediaType;
use crate::{LibraryDb, Result};
use log::warn;
use std::path::Path;
use std::time::SystemTime;
use walkdir::WalkDir;

const VIDEO_EXT: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "webm", "wmv", "flv", "m4v", "ts", "m2ts",
];
const AUDIO_EXT: &[&str] = &[
    "mp3", "flac", "wav", "aac", "ogg", "m4a", "opus", "wma", "aiff",
];
const BATCH_COMMIT_EVERY: usize = 64;

/// Whether `ext` (without dot) is a known media extension.
#[must_use]
pub fn is_media_extension(ext: &str) -> bool {
    let lower = ext.to_ascii_lowercase();
    VIDEO_EXT.contains(&lower.as_str()) || AUDIO_EXT.contains(&lower.as_str())
}

/// Whether `ext` (without dot) is a known video extension.
#[must_use]
pub fn is_video_extension(ext: &str) -> bool {
    VIDEO_EXT.contains(&ext.to_ascii_lowercase().as_str())
}

/// Whether `ext` (without dot) is a known audio extension.
#[must_use]
pub fn is_audio_extension(ext: &str) -> bool {
    AUDIO_EXT.contains(&ext.to_ascii_lowercase().as_str())
}

fn classify(ext: &str) -> MediaType {
    let lower = ext.to_ascii_lowercase();
    if VIDEO_EXT.contains(&lower.as_str()) {
        MediaType::Video
    } else if AUDIO_EXT.contains(&lower.as_str()) {
        MediaType::Audio
    } else {
        MediaType::Unknown
    }
}

/// Recursively scan `root` and upsert media into `db`.
///
/// `kind` is `video` or `audio` — tags the connected folder for that section
/// so Videos and Music libraries stay separate. Only files matching that kind
/// are indexed (video roots skip audio files and vice versa).
///
/// Returns the number of files upserted. Uses batched transactions.
///
/// # Errors
/// Propagates I/O and database errors.
pub fn scan_root(db: &LibraryDb, root: &Path, kind: &str) -> Result<usize> {
    let root_str = root.to_string_lossy();
    db.add_scan_root(&root_str, kind)?;
    let want = match kind {
        "audio" | "music" => Some(MediaType::Audio),
        "video" | "videos" => Some(MediaType::Video),
        _ => None,
    };
    let mut count = 0usize;
    let mut batch = 0usize;
    db.begin_write()?;
    let scan_result = (|| -> Result<usize> {
        for entry in WalkDir::new(root).follow_links(false).into_iter() {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("scan walk error under {}: {e}", root.display());
                    continue;
                }
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if !is_media_extension(ext) {
                continue;
            }
            let media_type = classify(ext);
            if let Some(w) = want {
                if media_type != w {
                    continue;
                }
            }
            let title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_owned();
            let mtime = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);
            let path_str = path.to_string_lossy();
            let duration_ms = probe_duration_ms(path);
            db.upsert_media(&path_str, &title, media_type, mtime, duration_ms)?;
            count += 1;
            batch += 1;
            if batch >= BATCH_COMMIT_EVERY {
                db.commit()?;
                db.begin_write()?;
                batch = 0;
            }
        }
        Ok(count)
    })();
    match scan_result {
        Ok(n) => {
            db.commit()?;
            let _ = db.prune_missing_media(&[root_str.to_string()]);
            Ok(n)
        }
        Err(e) => {
            let _ = db.rollback();
            Err(e)
        }
    }
}

/// Best-effort duration via ffprobe when available (scan stays fast if missing).
fn probe_duration_ms(path: &Path) -> Option<i64> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let secs: f64 = text.trim().parse().ok()?;
    if !secs.is_finite() || secs <= 0.0 {
        return None;
    }
    Some((secs * 1000.0).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_media_extension_should_accept_mp4_and_reject_txt() {
        assert!(is_media_extension("mp4"));
        assert!(is_media_extension("MP3"));
        assert!(!is_media_extension("txt"));
    }
}
