//! Cover art / video frame previews for library thumbnails.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::constants::{
    ARTWORK_FFMPEG_SEEK_SECS, ARTWORK_LARGE_FILE_BYTES, ARTWORK_MIN_BYTES, ARTWORK_VLC_START_LARGE,
    ARTWORK_VLC_START_SMALL,
};
use crate::process_util;
use log::debug;

/// DB / FE sentinel: generation was attempted and produced nothing (skip retries).
pub const NO_ARTWORK_SENTINEL: &str = ":none";

/// Resolve or generate a thumbnail for `media_path` under `cache_dir`.
///
/// Sidecar covers next to media are copied into `cache_dir` so they stay inside
/// the Tauri asset-protocol scope (`…/artwork/**`).
pub fn ensure_artwork(media_path: &Path, cache_dir: &Path, media_id: i64) -> Option<PathBuf> {
    let _ = fs::create_dir_all(cache_dir);

    if let Some(sidecar) = find_sidecar_cover(media_path) {
        if let Some(cached) = copy_sidecar_into_cache(&sidecar, cache_dir, media_id) {
            return Some(cached);
        }
    }

    let cached_jpg = cache_dir.join(format!("{media_id}.jpg"));
    let cached_png = cache_dir.join(format!("{media_id}.png"));
    for candidate in [&cached_jpg, &cached_png] {
        if candidate.is_file() {
            if let Ok(meta) = candidate.metadata() {
                if meta.len() > ARTWORK_MIN_BYTES {
                    return Some(candidate.clone());
                }
            }
        }
    }

    if is_audio(media_path) {
        if try_ffmpeg_embedded_cover(media_path, &cached_jpg) {
            return Some(cached_jpg);
        }
        if try_ffmpeg_embedded_cover(media_path, &cached_png) {
            return Some(cached_png);
        }
        return None;
    }

    if try_ffmpeg_frame(media_path, &cached_jpg) {
        return Some(cached_jpg);
    }
    if try_vlc_cli_frame(media_path, cache_dir, media_id) {
        for candidate in [&cached_jpg, &cached_png] {
            if candidate.is_file() {
                if let Ok(meta) = candidate.metadata() {
                    if meta.len() > ARTWORK_MIN_BYTES {
                        return Some(candidate.clone());
                    }
                }
            }
        }
    }
    None
}

fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(library::is_audio_extension)
}

fn copy_sidecar_into_cache(sidecar: &Path, cache_dir: &Path, media_id: i64) -> Option<PathBuf> {
    let ext = sidecar
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_ascii_lowercase();
    let dest = cache_dir.join(format!("{media_id}-cover.{ext}"));
    if dest.is_file() && file_nonempty(&dest) {
        // Refresh if sidecar is newer than the cached copy.
        let refresh = match (
            fs::metadata(sidecar).and_then(|m| m.modified()),
            fs::metadata(&dest).and_then(|m| m.modified()),
        ) {
            (Ok(src), Ok(dst)) => src > dst,
            _ => false,
        };
        if !refresh {
            return Some(dest);
        }
    }
    if fs::copy(sidecar, &dest).is_ok() && dest.is_file() && file_nonempty(&dest) {
        Some(dest)
    } else {
        None
    }
}

fn find_sidecar_cover(media_path: &Path) -> Option<PathBuf> {
    let dir = media_path.parent()?;
    // Prefer same-stem art first so a shared cover.jpg is not forced onto every file.
    if let Some(stem) = media_path.file_stem().and_then(|s| s.to_str()) {
        for ext in ["jpg", "jpeg", "png", "webp"] {
            let p = dir.join(format!("{stem}.{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
    }
    const NAMES: &[&str] = &[
        "cover.jpg",
        "cover.jpeg",
        "cover.png",
        "folder.jpg",
        "folder.png",
        "AlbumArt.jpg",
        "AlbumArt.png",
        "front.jpg",
        "Front.jpg",
    ];
    for name in NAMES {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn find_ffmpeg() -> Option<&'static Path> {
    static FFMPEG: OnceLock<Option<PathBuf>> = OnceLock::new();
    FFMPEG
        .get_or_init(|| {
            // Resolve once via `where` (CREATE_NO_WINDOW) — avoids PATH lookup per file.
            if let Ok(output) = process_util::command("where.exe")
                .arg("ffmpeg")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .output()
            {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout);
                    let first = text.lines().next().unwrap_or("").trim();
                    let p = PathBuf::from(first);
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
            // Last resort: bare name (CreateProcess PATH search). Cached either way.
            Some(PathBuf::from("ffmpeg"))
        })
        .as_deref()
}

fn find_vlc_exe() -> Option<&'static Path> {
    static VLC: OnceLock<Option<PathBuf>> = OnceLock::new();
    VLC.get_or_init(|| {
        let mut candidates = Vec::new();
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.join("vlc.exe"));
            }
        }
        #[cfg(debug_assertions)]
        {
            candidates
                .push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vendor/vlc/vlc.exe"));
        }
        if let Ok(pf) = std::env::var("ProgramFiles") {
            candidates.push(
                PathBuf::from(pf)
                    .join("VideoLAN")
                    .join("VLC")
                    .join("vlc.exe"),
            );
        }
        if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
            candidates.push(
                PathBuf::from(pf86)
                    .join("VideoLAN")
                    .join("VLC")
                    .join("vlc.exe"),
            );
        }
        candidates.into_iter().find(|p| p.is_file())
    })
    .as_deref()
}

fn try_ffmpeg_frame(media_path: &Path, out_jpg: &Path) -> bool {
    let Some(ffmpeg) = find_ffmpeg() else {
        return false;
    };
    let status = process_util::command(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            ARTWORK_FFMPEG_SEEK_SECS,
            "-i",
        ])
        .arg(media_path)
        .args(["-frames:v", "1", "-q:v", "4", "-y"])
        .arg(out_jpg)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    matches!(status, Ok(s) if s.success() && out_jpg.is_file() && file_nonempty(out_jpg))
}

/// Extract embedded album art (attached pic / cover stream) from audio files.
fn try_ffmpeg_embedded_cover(media_path: &Path, out: &Path) -> bool {
    let Some(ffmpeg) = find_ffmpeg() else {
        return false;
    };
    let status = process_util::command(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(media_path)
        .args(["-an", "-vcodec", "copy", "-y"])
        .arg(out)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    matches!(status, Ok(s) if s.success() && out.is_file() && file_nonempty(out))
}

fn file_nonempty(p: &Path) -> bool {
    fs::metadata(p)
        .map(|m| m.len() > ARTWORK_MIN_BYTES)
        .unwrap_or(false)
}

fn try_vlc_cli_frame(media_path: &Path, cache_dir: &Path, media_id: i64) -> bool {
    let Some(vlc) = find_vlc_exe() else {
        debug!("vlc.exe not found for thumbnails");
        return false;
    };
    let scene_dir = cache_dir.join(format!("vlc-scene-{media_id}"));
    let _ = fs::remove_dir_all(&scene_dir);
    if fs::create_dir_all(&scene_dir).is_err() {
        return false;
    }
    let start = if media_path
        .metadata()
        .map(|m| m.len() > ARTWORK_LARGE_FILE_BYTES)
        .unwrap_or(false)
    {
        ARTWORK_VLC_START_LARGE
    } else {
        ARTWORK_VLC_START_SMALL
    };
    let status = process_util::command(vlc)
        .args([
            "-I",
            "dummy",
            "--quiet",
            "--no-video-title-show",
            "--no-audio",
            "--no-osd",
            "--play-and-exit",
            start,
            "--run-time=1",
            "--video-filter=scene",
            "--scene-format=jpg",
            "--scene-ratio=1",
            "--scene-prefix",
            "shot",
            "--scene-replace",
        ])
        .arg(format!("--scene-path={}", scene_dir.display()))
        .arg(media_path)
        .arg("vlc://quit")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let _ = status;
    let mut found: Option<PathBuf> = None;
    if let Ok(rd) = fs::read_dir(&scene_dir) {
        let mut jpgs: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("jpg"))
            .collect();
        jpgs.sort();
        found = jpgs.pop();
    }
    let Some(src) = found else {
        let _ = fs::remove_dir_all(&scene_dir);
        return false;
    };
    let dest = cache_dir.join(format!("{media_id}.jpg"));
    let copied = fs::copy(&src, &dest).is_ok();
    let _ = fs::remove_dir_all(&scene_dir);
    copied && dest.is_file() && file_nonempty(&dest)
}
